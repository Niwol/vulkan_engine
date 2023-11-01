use std::sync::Arc;
use std::{collections::BTreeMap, f32::consts::PI};

use smallvec::smallvec;

use vulkano::buffer::Buffer;
use vulkano::descriptor_set::WriteDescriptorSet;
use vulkano::{
    buffer::{BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo},
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
        RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, SubpassEndInfo,
    },
    descriptor_set::layout::{
        DescriptorBindingFlags, DescriptorSetLayout, DescriptorSetLayoutBinding,
        DescriptorSetLayoutCreateFlags, DescriptorSetLayoutCreateInfo, DescriptorType,
    },
    device::Device,
    device::Queue,
    format::{ClearValue, Format},
    image::{
        sampler::ComponentMapping,
        view::{ImageView, ImageViewCreateInfo, ImageViewType},
        Image, ImageAspects, ImageLayout, ImageSubresourceRange, ImageUsage, SampleCount,
    },
    memory::allocator::{
        AllocationCreateInfo, MemoryAllocatePreference, MemoryTypeFilter, StandardMemoryAllocator,
    },
    pipeline::{
        graphics::{
            color_blend::{
                ColorBlendAttachmentState, ColorBlendState, ColorBlendStateFlags, ColorComponents,
            },
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::{
                CullMode, FrontFace, LineRasterizationMode, PolygonMode, RasterizationState,
            },
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Scissor, Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::{PipelineLayoutCreateFlags, PipelineLayoutCreateInfo},
        GraphicsPipeline, PipelineBindPoint, PipelineCreateFlags, PipelineLayout,
        PipelineShaderStageCreateInfo,
    },
    render_pass::{
        AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp,
        Framebuffer, FramebufferCreateInfo, RenderPass, RenderPassCreateInfo, Subpass,
        SubpassDescription,
    },
    shader::ShaderStages,
    swapchain::{
        self, ColorSpace, CompositeAlpha, FullScreenExclusive, PresentMode, SurfaceCapabilities,
        SurfaceInfo, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
    },
    sync::{GpuFuture, Sharing},
};

use winit::window::Window;

use glam::{Mat4, Vec3};

use super::{
    render_object::{RenderObject, Vertex as MyVertex},
    Engine,
};

mod shaders {
    vulkano_shaders::shader! {
        shaders: {
            vertex: {
                ty: "vertex",
                path: "shaders/shader.vert",
            },

            fragment: {
                ty: "fragment",
                path: "shaders/shader.frag",
            },
        },
    }
}

#[derive(BufferContents)]
#[repr(C)]
struct MVP {
    model: Mat4,
    view: Mat4,
    projection: Mat4,
}

pub struct Renderer {
    _device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    present_queue: Arc<Queue>,

    swapchain: Arc<Swapchain>,
    _swapchain_images: Vec<Arc<Image>>,
    _swapchain_image_views: Vec<Arc<ImageView>>,

    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    graphic_pipeline: Arc<GraphicsPipeline>,
    pipeline_layout: Arc<PipelineLayout>,

    command_buffer_allocator: StandardCommandBufferAllocator,

    mvp_buffer: Subbuffer<[MVP]>,
}

impl Renderer {
    pub(crate) fn new(engine: &Engine) -> Self {
        let device = engine.device();

        let graphics_queue = engine.graphics_queue();
        let present_queue = engine.present_queue();

        let (swapchain, swapchain_images) = Self::create_swapchain(engine);
        let swapchain_image_views =
            Self::create_swapchain_image_views(&swapchain, &swapchain_images);

        let render_pass = Self::create_render_pass(&device, &swapchain);
        let framebuffers =
            Self::create_framebuffers(&render_pass, &swapchain, &swapchain_image_views);

        let (graphic_pipeline, pipeline_layout) =
            Self::create_graphic_pipeline(&device, &swapchain, &render_pass);

        let mvp_buffer = Self::create_mvp_buffer(&engine.standard_memory_allocator());

        let command_buffer_allocator = Self::create_command_buffer_allocator(&device);

        Self {
            _device: device.clone(),
            graphics_queue,
            present_queue,

            swapchain,
            _swapchain_images: swapchain_images,
            _swapchain_image_views: swapchain_image_views,

            render_pass,
            framebuffers,
            graphic_pipeline,
            pipeline_layout,

            command_buffer_allocator,
            mvp_buffer,
        }
    }

    pub fn draw_frame(&self, render_object: &RenderObject) {
        let (image_index, _suboptimal, acquire_future) =
            swapchain::acquire_next_image(self.swapchain.clone(), None)
                .expect("Failed to acquire next image");

        let command_buffer = self.record_draw_command_buffer(image_index as usize, render_object);

        let _ = acquire_future
            .then_execute(self.graphics_queue.clone(), command_buffer)
            .expect("Failed to execute draw command buffer")
            .then_swapchain_present(
                self.present_queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush()
            .expect("Failed to signal fence");
    }

    fn get_minimum_image_count(capabilities: &SurfaceCapabilities) -> u32 {
        if let Some(max_image_count) = capabilities.max_image_count {
            if max_image_count == capabilities.min_image_count {
                return max_image_count;
            }
        }

        capabilities.min_image_count + 1
    }

    fn choose_swapchain_format(
        available_formats: Vec<(Format, ColorSpace)>,
    ) -> (Format, ColorSpace) {
        for (format, color_space) in available_formats.iter() {
            if *format == Format::R8G8B8A8_SRGB && *color_space == ColorSpace::SrgbNonLinear {
                return (*format, *color_space);
            }
        }

        available_formats[0]
    }

    fn choose_swapchain_extent(
        window: &Arc<Window>,
        capabilities: &SurfaceCapabilities,
    ) -> [u32; 2] {
        if let Some(extent) = capabilities.current_extent {
            return extent;
        }

        let window_dimensions = window.outer_size();

        let extent = [
            window_dimensions.width.clamp(
                capabilities.min_image_extent[0],
                capabilities.max_image_extent[0],
            ),
            window_dimensions.height.clamp(
                capabilities.min_image_extent[1],
                capabilities.max_image_extent[1],
            ),
        ];

        extent
    }

    fn choose_present_mode(available_present_modes: Vec<PresentMode>) -> PresentMode {
        for present_mode in available_present_modes.iter() {
            if *present_mode == PresentMode::Mailbox {
                return *present_mode;
            }
        }

        PresentMode::Fifo
    }

    fn create_swapchain(engine: &Engine) -> (Arc<Swapchain>, Vec<Arc<Image>>) {
        let device = engine.device();
        let physical_device = device.physical_device();

        let surface_info = SurfaceInfo {
            full_screen_exclusive: FullScreenExclusive::Default,
            ..Default::default()
        };

        let surface = engine.window_surface();

        let surface_capabilities = physical_device
            .surface_capabilities(surface.as_ref(), surface_info.clone())
            .expect("Failed to get surface capabilities");

        let available_formats = physical_device
            .surface_formats(surface.as_ref(), surface_info.clone())
            .expect("Failed to get surface formats");

        let (format, color_space) = Self::choose_swapchain_format(available_formats);
        let extent = Self::choose_swapchain_extent(&engine.window(), &surface_capabilities);

        let sharing = if engine.graphics_queue().queue_family_index()
            == engine.present_queue().queue_family_index()
        {
            Sharing::Exclusive
        } else {
            todo!()
        };

        let available_present_modes = physical_device
            .surface_present_modes(surface.as_ref(), surface_info)
            .expect("Failed to get supported present modes")
            .collect();
        let present_mode = Self::choose_present_mode(available_present_modes);

        let swapchain_info = SwapchainCreateInfo {
            min_image_count: Self::get_minimum_image_count(&surface_capabilities),
            image_format: format,
            image_color_space: color_space,
            image_extent: extent,
            image_array_layers: 1,
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            image_sharing: sharing,
            pre_transform: surface_capabilities.current_transform,
            composite_alpha: CompositeAlpha::Opaque,
            present_mode,
            clipped: true,
            ..Default::default()
        };

        Swapchain::new(device.clone(), surface.clone(), swapchain_info)
            .expect("Failed to create swapchain")
    }

    fn create_swapchain_image_views(
        swapchain: &Arc<Swapchain>,
        swapchain_images: &Vec<Arc<Image>>,
    ) -> Vec<Arc<ImageView>> {
        let mut image_views = Vec::new();

        for image in swapchain_images.iter() {
            let view_info = ImageViewCreateInfo {
                view_type: ImageViewType::Dim2d,
                format: swapchain.image_format(),
                component_mapping: ComponentMapping::identity(),
                subresource_range: ImageSubresourceRange {
                    aspects: ImageAspects::COLOR,
                    mip_levels: 0..1,
                    array_layers: 0..1,
                },
                usage: ImageUsage::COLOR_ATTACHMENT,
                ..Default::default()
            };

            image_views.push(
                ImageView::new(image.clone(), view_info).expect("Failed to create image view"),
            );
        }

        image_views
    }

    fn create_framebuffers(
        render_pass: &Arc<RenderPass>,
        swapchain: &Arc<Swapchain>,
        image_views: &Vec<Arc<ImageView>>,
    ) -> Vec<Arc<Framebuffer>> {
        let mut framebuffers = Vec::new();

        for image_view in image_views.iter() {
            let framebuffer_info = FramebufferCreateInfo {
                attachments: vec![image_view.clone()],
                extent: swapchain.image_extent(),
                layers: 1,
                ..Default::default()
            };

            framebuffers.push(
                Framebuffer::new(render_pass.clone(), framebuffer_info)
                    .expect("Failed to create framebuffer"),
            );
        }

        framebuffers
    }

    fn create_render_pass(device: &Arc<Device>, swapchain: &Arc<Swapchain>) -> Arc<RenderPass> {
        let color_attachment = AttachmentDescription {
            format: swapchain.image_format(),
            samples: SampleCount::Sample1,
            load_op: AttachmentLoadOp::Clear,
            store_op: AttachmentStoreOp::Store,
            stencil_load_op: Some(AttachmentLoadOp::DontCare),
            stencil_store_op: Some(AttachmentStoreOp::DontCare),
            initial_layout: ImageLayout::Undefined,
            final_layout: ImageLayout::PresentSrc,
            ..Default::default()
        };

        let color_attachment_ref = AttachmentReference {
            attachment: 0,
            layout: ImageLayout::ColorAttachmentOptimal,
            ..Default::default()
        };

        let color_subpass = SubpassDescription {
            view_mask: 0,
            color_attachments: vec![Some(color_attachment_ref)],
            ..Default::default()
        };

        let attachments = vec![color_attachment];
        let subpasses = vec![color_subpass];
        let dependencies = vec![];

        let render_pass_info = RenderPassCreateInfo {
            attachments,
            subpasses,
            dependencies,
            ..Default::default()
        };

        RenderPass::new(device.clone(), render_pass_info).expect("Failed to create render pass")
    }

    fn create_pipeline_layout(device: &Arc<Device>) -> Arc<PipelineLayout> {
        let mut mvp_bindings = BTreeMap::new();
        mvp_bindings.insert(
            0,
            DescriptorSetLayoutBinding {
                binding_flags: DescriptorBindingFlags::empty(),
                descriptor_count: 1,
                stages: ShaderStages::VERTEX,
                immutable_samplers: Vec::new(),
                ..DescriptorSetLayoutBinding::descriptor_type(DescriptorType::UniformBuffer)
            },
        );

        let mvp_info = DescriptorSetLayoutCreateInfo {
            flags: DescriptorSetLayoutCreateFlags::PUSH_DESCRIPTOR,
            bindings: mvp_bindings,
            ..Default::default()
        };

        let mvp_descriptor_set = DescriptorSetLayout::new(device.clone(), mvp_info)
            .expect("Failed to create descriptor set layout");

        let layout_info = PipelineLayoutCreateInfo {
            flags: PipelineLayoutCreateFlags::empty(),
            set_layouts: vec![mvp_descriptor_set],
            push_constant_ranges: Vec::new(),
            ..Default::default()
        };

        PipelineLayout::new(device.clone(), layout_info).expect("Failed to create pipeline layout")
    }

    fn create_graphic_pipeline(
        device: &Arc<Device>,
        swapchain: &Arc<Swapchain>,
        render_pass: &Arc<RenderPass>,
    ) -> (Arc<GraphicsPipeline>, Arc<PipelineLayout>) {
        let vertex_shader = shaders::load_vertex(device.clone())
            .expect("Failed to load vertex shader")
            .entry_point("main")
            .unwrap();
        let fragment_shader = shaders::load_fragment(device.clone())
            .expect("Failed to load fragment shader")
            .entry_point("main")
            .unwrap();

        let window_dimensions = swapchain.image_extent();
        let window_dimensions_f32 = [window_dimensions[0] as f32, window_dimensions[1] as f32];

        let viewport = ViewportState {
            viewports: smallvec![Viewport {
                offset: [0.0, 0.0],
                extent: window_dimensions_f32,
                depth_range: 0.0..=1.0,
            }],
            scissors: smallvec![Scissor {
                offset: [0, 0],
                extent: window_dimensions
            }],
            ..Default::default()
        };

        let vertex_input_state = MyVertex::per_vertex()
            .definition(&vertex_shader.info().input_interface)
            .expect("Failed to get vertex input state");

        let pipeline_layout = Self::create_pipeline_layout(device);

        let pipeline_info = GraphicsPipelineCreateInfo {
            flags: PipelineCreateFlags::empty(),
            stages: smallvec![
                PipelineShaderStageCreateInfo::new(vertex_shader),
                PipelineShaderStageCreateInfo::new(fragment_shader),
            ],
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState {
                topology: PrimitiveTopology::TriangleList,
                primitive_restart_enable: false,
                ..Default::default()
            }),
            tessellation_state: None,
            viewport_state: Some(viewport),
            rasterization_state: Some(RasterizationState {
                depth_clamp_enable: false,
                rasterizer_discard_enable: false,
                polygon_mode: PolygonMode::Fill,
                cull_mode: CullMode::Back,
                front_face: FrontFace::Clockwise,
                depth_bias: None,
                line_width: 1.0,
                line_rasterization_mode: LineRasterizationMode::Default,
                line_stipple: None,
                ..Default::default()
            }),
            multisample_state: Some(MultisampleState::default()),
            depth_stencil_state: None,
            color_blend_state: Some(ColorBlendState {
                flags: ColorBlendStateFlags::empty(),
                logic_op: None,
                attachments: vec![ColorBlendAttachmentState {
                    blend: None,
                    color_write_mask: ColorComponents::all(),
                    color_write_enable: true,
                }],
                blend_constants: [0.0; 4],
                ..Default::default()
            }),
            subpass: Some(Subpass::from(render_pass.clone(), 0).unwrap().into()),
            discard_rectangle_state: None,
            ..GraphicsPipelineCreateInfo::layout(pipeline_layout.clone())
        };

        let pipeline = GraphicsPipeline::new(device.clone(), None, pipeline_info)
            .expect("Failed to create graphic pipeline");

        (pipeline, pipeline_layout)
    }

    fn create_command_buffer_allocator(device: &Arc<Device>) -> StandardCommandBufferAllocator {
        let allocator_info = StandardCommandBufferAllocatorCreateInfo {
            primary_buffer_count: 16,
            secondary_buffer_count: 0,
            ..Default::default()
        };

        StandardCommandBufferAllocator::new(device.clone(), allocator_info)
    }

    fn create_mvp_buffer(allocator: &Arc<StandardMemoryAllocator>) -> Subbuffer<[MVP]> {
        let buffer_info = BufferCreateInfo {
            sharing: Sharing::Exclusive, // TODO: handle sharing accross different queues
            usage: BufferUsage::UNIFORM_BUFFER,
            ..Default::default()
        };

        let allocation_info = AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            allocate_preference: MemoryAllocatePreference::Unknown,
            ..Default::default()
        };

        let mut mvp = MVP {
            model: Mat4::IDENTITY,
            view: Mat4::look_to_rh(
                Vec3::new(0.0, 1.0, 3.0),
                Vec3::new(0.0, -0.2, -1.0),
                Vec3::new(0.0, 1.0, 0.0),
            ),
            projection: Mat4::perspective_rh(f32::to_radians(45.0), 800.0 / 600.0, 0.1, 100.0),
        };

        mvp.projection.as_mut()[1 * 4 + 1] *= -1.0;

        Buffer::from_iter(allocator.clone(), buffer_info, allocation_info, [mvp])
            .expect("Failed to create mvp buffer")
    }

    fn record_draw_command_buffer(
        &self,
        image_index: usize,
        render_object: &RenderObject,
    ) -> Arc<PrimaryAutoCommandBuffer> {
        let render_pass_begin_info = RenderPassBeginInfo {
            render_pass: self.render_pass.clone(),
            render_area_offset: [0, 0],
            render_area_extent: self.swapchain.image_extent(),
            clear_values: vec![Some(ClearValue::Float([0.0, 0.0, 0.0, 1.0]))],
            ..RenderPassBeginInfo::framebuffer(self.framebuffers[image_index].clone())
        };

        let subpass_begin_info = SubpassBeginInfo {
            contents: SubpassContents::Inline,
            ..Default::default()
        };

        let subpass_end_info = SubpassEndInfo {
            ..Default::default()
        };

        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.graphics_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .expect("Failed to start recording command buffer");

        let vertex_buffer = render_object.vectex_buffer();
        let index_buffer = render_object.index_buffer();

        builder
            .begin_render_pass(render_pass_begin_info, subpass_begin_info)
            .expect("Failed to begin render pass command")
            .bind_pipeline_graphics(self.graphic_pipeline.clone())
            .expect("Failed to bind graphics pipeline")
            .bind_vertex_buffers(0, vertex_buffer.clone())
            .expect("Failed to bind vertex buffer")
            .bind_index_buffer(index_buffer.clone())
            .expect("Failed to bind index buffer")
            .push_descriptor_set(PipelineBindPoint::Graphics, self.pipeline_layout.clone(), 0, smallvec![WriteDescriptorSet::buffer(0, self.mvp_buffer.clone())])
            .expect("Failed to bind descriptor set")
            .draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0)
            .expect("Failed draw command")
            .end_render_pass(subpass_end_info)
            .expect("Failed end render pass command");

        let command_buffer = builder.build().expect("Failed to build command buffer");

        command_buffer
    }
}
