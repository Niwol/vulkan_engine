use std::sync::Arc;

use vulkano::{
    command_buffer::{
        allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo},
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
        RenderPassBeginInfo, SubpassContents,
    },
    device::Device,
    format::{ClearValue, Format},
    image::{
        view::{ImageView, ImageViewCreateInfo},
        ImageAspects, ImageLayout, ImageSubresourceRange, ImageUsage, ImageViewType, SampleCount,
        SwapchainImage,
    },
    pipeline::{
        graphics::viewport::{Scissor, Viewport, ViewportState},
        GraphicsPipeline,
    },
    render_pass::{
        AttachmentDescription, AttachmentReference, Framebuffer, FramebufferCreateInfo, LoadOp,
        RenderPass, RenderPassCreateInfo, StoreOp, Subpass, SubpassDescription,
    },
    sampler::ComponentMapping,
    swapchain::{
        self, ColorSpace, CompositeAlpha, FullScreenExclusive, PresentMode, Surface,
        SurfaceCapabilities, SurfaceInfo, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
    },
    sync::{GpuFuture, Sharing},
};
use winit::window::Window;

use super::Queues;

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

pub struct Renderer {
    _device: Arc<Device>,
    queues: Queues,

    swapchain: Arc<Swapchain>,
    _swapchain_images: Vec<Arc<SwapchainImage>>,
    _swapchain_image_views: Vec<Arc<ImageView<SwapchainImage>>>,

    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    graphic_pipeline: Arc<GraphicsPipeline>,

    command_buffer_allocator: StandardCommandBufferAllocator,
}

impl Renderer {
    pub(crate) fn new(
        device: &Arc<Device>,
        surface: &Arc<Surface>,
        window: &Arc<Window>,
        queues: &Queues,
    ) -> Self {
        let queues = Queues {
            graphic_queue: queues.graphic_queue.clone(),
            present_queue: queues.present_queue.clone(),
        };

        let (swapchain, swapchain_images) =
            Self::create_swapchain(device, surface, window, &queues);
        let swapchain_image_views =
            Self::create_swapchain_image_views(&swapchain, &swapchain_images);

        let render_pass = Self::create_render_pass(device, &swapchain);
        let framebuffers =
            Self::create_framebuffers(&render_pass, &swapchain, &swapchain_image_views);

        let graphic_pipeline = Self::create_graphic_pipeline(device, &swapchain, &render_pass);

        let command_buffer_allocator = Self::create_command_buffer_allocator(device);

        Renderer {
            _device: device.clone(),
            queues,

            swapchain,
            _swapchain_images: swapchain_images,
            _swapchain_image_views: swapchain_image_views,

            render_pass,
            framebuffers,
            graphic_pipeline,

            command_buffer_allocator,
        }
    }

    pub fn draw_frame(&self) {
        let (image_index, _suboptimal, acquire_future) =
            swapchain::acquire_next_image(self.swapchain.clone(), None)
                .expect("Failed to acquire next image");

        let command_buffer = self.record_draw_command_buffer(image_index as usize);

        let _ = acquire_future
            .then_execute(self.queues.graphic_queue.clone(), command_buffer)
            .expect("Failed to execute draw command buffer")
            .then_swapchain_present(
                self.queues.present_queue.clone(),
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

    fn create_swapchain(
        device: &Arc<Device>,
        surface: &Arc<Surface>,
        window: &Arc<Window>,
        queues: &Queues,
    ) -> (Arc<Swapchain>, Vec<Arc<SwapchainImage>>) {
        let physical_device = device.physical_device();

        let surface_info = SurfaceInfo {
            full_screen_exclusive: FullScreenExclusive::Default,
            ..Default::default()
        };

        let surface_capabilities = physical_device
            .surface_capabilities(surface.as_ref(), surface_info.clone())
            .expect("Failed to get surface capabilities");

        let available_formats = physical_device
            .surface_formats(surface.as_ref(), surface_info)
            .expect("Failed to get surface formats");

        let (format, color_space) = Self::choose_swapchain_format(available_formats);
        let extent = Self::choose_swapchain_extent(window, &surface_capabilities);

        let sharing = if queues.graphic_queue.queue_family_index()
            == queues.present_queue.queue_family_index()
        {
            Sharing::Exclusive
        } else {
            todo!()
        };

        let available_present_modes = physical_device
            .surface_present_modes(surface.as_ref())
            .expect("Failed to get supported present modes")
            .collect();
        let present_mode = Self::choose_present_mode(available_present_modes);

        let swapchain_info = SwapchainCreateInfo {
            min_image_count: Self::get_minimum_image_count(&surface_capabilities),
            image_format: Some(format),
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
        swapchain_images: &Vec<Arc<SwapchainImage>>,
    ) -> Vec<Arc<ImageView<SwapchainImage>>> {
        let mut image_views = Vec::new();

        for image in swapchain_images.iter() {
            let view_info = ImageViewCreateInfo {
                view_type: ImageViewType::Dim2d,
                format: Some(swapchain.image_format()),
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
        image_views: &Vec<Arc<ImageView<SwapchainImage>>>,
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
            format: Some(swapchain.image_format()),
            samples: SampleCount::Sample1,
            load_op: LoadOp::Clear,
            store_op: StoreOp::Store,
            stencil_load_op: LoadOp::DontCare,
            stencil_store_op: StoreOp::DontCare,
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

    fn create_graphic_pipeline(
        device: &Arc<Device>,
        swapchain: &Arc<Swapchain>,
        render_pass: &Arc<RenderPass>,
    ) -> Arc<GraphicsPipeline> {
        let vertex_shader =
            shaders::load_vertex(device.clone()).expect("Failed to load vertex shader");
        let fragment_shader =
            shaders::load_fragment(device.clone()).expect("Failed to load fragment shader");

        let window_dimensions = swapchain.image_extent();
        let window_dimensions_f32 = [window_dimensions[0] as f32, window_dimensions[1] as f32];

        let viewport = ViewportState::Fixed {
            data: vec![(
                Viewport {
                    origin: [0.0, 0.0],
                    dimensions: window_dimensions_f32,
                    depth_range: (0.0)..(1.0),
                },
                Scissor {
                    origin: [0, 0],
                    dimensions: window_dimensions,
                },
            )],
        };

        GraphicsPipeline::start()
            .vertex_shader(vertex_shader.entry_point("main").unwrap(), ())
            .fragment_shader(fragment_shader.entry_point("main").unwrap(), ())
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .viewport_state(viewport)
            .build(device.clone())
            .expect("Failed to build graphics pipeline")
    }

    fn create_command_buffer_allocator(device: &Arc<Device>) -> StandardCommandBufferAllocator {
        let allocator_info = StandardCommandBufferAllocatorCreateInfo {
            primary_buffer_count: 16,
            secondary_buffer_count: 0,
            ..Default::default()
        };

        StandardCommandBufferAllocator::new(device.clone(), allocator_info)
    }

    fn record_draw_command_buffer(&self, image_index: usize) -> PrimaryAutoCommandBuffer {
        let render_pass_begin_info = RenderPassBeginInfo {
            render_pass: self.render_pass.clone(),
            render_area_offset: [0, 0],
            render_area_extent: self.swapchain.image_extent(),
            clear_values: vec![Some(ClearValue::Float([0.0, 0.0, 0.0, 1.0]))],
            ..RenderPassBeginInfo::framebuffer(self.framebuffers[image_index].clone())
        };

        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.queues.graphic_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .expect("Failed to start recording command buffer");

        builder
            .begin_render_pass(render_pass_begin_info, SubpassContents::Inline)
            .expect("Failed to begin render pass command")
            .bind_pipeline_graphics(self.graphic_pipeline.clone())
            .draw(3, 1, 0, 0)
            .expect("Failed draw command")
            .end_render_pass()
            .expect("Failed end render pass command");

        let command_buffer = builder.build().expect("Failed to build command buffer");

        command_buffer
    }
}
