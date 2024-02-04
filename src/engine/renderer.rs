use std::mem::size_of;
use std::sync::Arc;

use anyhow::Result;

use vulkano::{
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
        RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, SubpassEndInfo,
    },
    descriptor_set::DescriptorSetWithOffsets,
    device::Device,
    format::{ClearValue, Format},
    image::{
        sampler::ComponentMapping,
        view::{ImageView, ImageViewCreateInfo, ImageViewType},
        Image, ImageAspects, ImageCreateInfo, ImageLayout, ImageSubresourceRange, ImageType,
        ImageUsage, SampleCount,
    },
    memory::allocator::{AllocationCreateInfo, MemoryAllocatePreference, MemoryTypeFilter},
    pipeline::{
        graphics::viewport::{Scissor, Viewport},
        Pipeline, PipelineBindPoint,
    },
    render_pass::{
        AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp,
        Framebuffer, FramebufferCreateInfo, RenderPass, RenderPassCreateInfo, SubpassDescription,
    },
    swapchain::{
        self, ColorSpace, CompositeAlpha, FullScreenExclusive, PresentMode, Surface,
        SurfaceCapabilities, SurfaceInfo, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
    },
    sync::{GpuFuture, Sharing},
    Validated, VulkanError,
};

use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    engine::{
        ecs::Scene,
        material::material_manager::MaterialManager,
        pipeline_manager::{PipelineManager, VulkanPipeline},
    },
    vulkan_context::VulkanContext,
};

use super::ecs::components::MeshComponent;

#[derive(Debug, Clone, Copy)]
pub enum RenderMode {
    Default,
    NormalView,
    DepthView,
}

pub struct Renderer {
    vulkan_context: Arc<VulkanContext>,
    window: Arc<Window>,

    swapchain: Arc<Swapchain>,
    _swapchain_images: Vec<Arc<Image>>,
    _swapchain_image_views: Vec<Arc<ImageView>>,

    depth_image: Arc<Image>,
    depth_image_view: Arc<ImageView>,

    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,

    pipeline_manager: PipelineManager,

    render_mode: RenderMode,
}

impl Renderer {
    pub(crate) fn new(
        vulkan_context: Arc<VulkanContext>,
        window: Arc<Window>,
        material_manager: &MaterialManager,
    ) -> Result<Self> {
        let device = vulkan_context.device();

        let (swapchain, swapchain_images) = Self::create_swapchain(&vulkan_context, &window)?;
        let swapchain_image_views =
            Self::create_swapchain_image_views(&swapchain, &swapchain_images)?;

        let image_extent = swapchain.image_extent();
        let (depth_image, depth_image_view) =
            Self::create_depth_image(&vulkan_context, image_extent)?;

        let render_pass = Self::create_render_pass(&device, &swapchain, &depth_image);
        let framebuffers = Self::create_framebuffers(
            &render_pass,
            &swapchain,
            &swapchain_image_views,
            &depth_image_view,
        )?;

        let pipeline_manager = PipelineManager::new(
            &vulkan_context,
            &render_pass,
            Arc::clone(material_manager.material_set_layout()),
        )?;

        Ok(Self {
            vulkan_context,
            window,

            swapchain,
            _swapchain_images: swapchain_images,
            _swapchain_image_views: swapchain_image_views,

            depth_image,
            depth_image_view,

            render_pass,
            framebuffers,
            pipeline_manager,

            render_mode: RenderMode::Default,
        })
    }

    pub(crate) fn _set_render_mode(&mut self, render_mode: RenderMode) {
        self.render_mode = render_mode;
    }

    pub fn clear_screen(&self) -> Result<()> {
        todo!("Rendering currently clears automaticaly => TODO: Handle rendering without clearing");
    }

    pub(crate) fn render_scene(&mut self, scene: &Scene) -> Result<()> {
        debug_assert!(scene.camera().is_some());

        let (image_index, _suboptimal, swapchain_future) =
            match swapchain::acquire_next_image(self.swapchain.clone(), None)
                .map_err(Validated::unwrap)
            {
                Ok(x) => x,
                Err(vulkano::VulkanError::OutOfDate) => panic!(),
                Err(e) => panic!("{e}"),
            };

        let command_buffer = match self.render_mode {
            RenderMode::Default => self.record_draw_command_buffer(
                image_index as usize,
                scene,
                self.pipeline_manager.material_pipeline(),
            )?,
            RenderMode::NormalView => self.record_debug_draw_command_buffer(
                image_index as usize,
                scene,
                self.pipeline_manager.normal_pipeline(),
            )?,
            RenderMode::DepthView => self.record_debug_draw_command_buffer(
                image_index as usize,
                scene,
                self.pipeline_manager.depth_pipeline(),
            )?,
        };

        let future = swapchain_future
            .then_execute(
                Arc::clone(self.vulkan_context.graphics_queue()),
                command_buffer,
            )?
            .then_swapchain_present(
                Arc::clone(self.vulkan_context.present_queue()),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        match future.map_err(Validated::unwrap) {
            Ok(_) => (),

            Err(VulkanError::OutOfDate) => {
                self.resize(self.window.inner_size())?;
            }

            Err(e) => panic!("{:#?}", e),
        }

        Ok(())
    }

    fn record_draw_command_buffer(
        &self,
        image_index: usize,
        scene: &Scene,
        vulkan_pipeline: &VulkanPipeline,
    ) -> Result<Arc<PrimaryAutoCommandBuffer>> {
        let pipeline = &vulkan_pipeline.pipeline;
        let layout = &vulkan_pipeline.layout;
        let camera = scene.camera().as_ref().unwrap();

        let render_pass_begin_info = RenderPassBeginInfo {
            render_pass: self.render_pass.clone(),
            render_area_offset: [0, 0],
            render_area_extent: self.swapchain.image_extent(),
            clear_values: vec![
                Some(ClearValue::Float([0.5, 0.5, 0.5, 1.0])),
                Some(ClearValue::Depth(1.0)),
            ],
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
            self.vulkan_context
                .standard_command_buffer_allocator()
                .as_ref(),
            self.vulkan_context.graphics_queue().queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )?;

        let [width, height] = self.swapchain.image_extent().map(|x| x as f32);
        let mut projection =
            glam::Mat4::perspective_rh(f32::to_radians(45.0), width / height, 0.1, 100.0);
        projection.as_mut()[1 * 4 + 1] *= -1.0;

        builder
            .begin_render_pass(render_pass_begin_info, subpass_begin_info)?
            .bind_pipeline_graphics(Arc::clone(pipeline))?
            .push_constants(
                Arc::clone(layout),
                16 * size_of::<f32>() as u32,
                camera.get_view(),
            )?
            .push_constants(
                Arc::clone(layout),
                2 * 16 * size_of::<f32>() as u32,
                projection,
            )?
            .set_viewport(
                0,
                [Viewport {
                    offset: [0.0, 0.0],
                    extent: self.swapchain.image_extent().map(|x| x as f32),
                    depth_range: 0.0..=1.0,
                }]
                .into_iter()
                .collect(),
            )?
            .set_scissor(
                0,
                [Scissor {
                    offset: [0, 0],
                    extent: self.swapchain.image_extent(),
                }]
                .into_iter()
                .collect(),
            )?;

        for (_, mesh_component) in scene.components::<MeshComponent>().unwrap() {
            let vertex_buffer = mesh_component.mesh.vectex_buffer();
            let index_buffer = mesh_component.mesh.index_buffer();
            let material_descriptor_set = Arc::clone(
                scene
                    .material_manager()
                    .descriptor_set(mesh_component.material),
            );

            builder
                .bind_vertex_buffers(0, vertex_buffer.clone())?
                .bind_index_buffer(index_buffer.clone())?
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    Arc::clone(pipeline.layout()),
                    0,
                    vec![DescriptorSetWithOffsets::new(material_descriptor_set, [])],
                )?
                .push_constants(Arc::clone(layout), 0, mesh_component.model.transform())?
                .draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0)?;
        }

        builder.end_render_pass(subpass_end_info)?;

        let command_buffer = builder.build()?;

        Ok(command_buffer)
    }

    fn record_debug_draw_command_buffer(
        &self,
        image_index: usize,
        scene: &Scene,
        vulkan_pipeline: &VulkanPipeline,
    ) -> Result<Arc<PrimaryAutoCommandBuffer>> {
        let pipeline = &vulkan_pipeline.pipeline;
        let layout = &vulkan_pipeline.layout;
        let camera = scene.camera().as_ref().unwrap();

        let render_pass_begin_info = RenderPassBeginInfo {
            render_pass: self.render_pass.clone(),
            render_area_offset: [0, 0],
            render_area_extent: self.swapchain.image_extent(),
            clear_values: vec![
                Some(ClearValue::Float([0.5, 0.5, 0.5, 1.0])),
                Some(ClearValue::Depth(1.0)),
            ],
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
            self.vulkan_context
                .standard_command_buffer_allocator()
                .as_ref(),
            self.vulkan_context.graphics_queue().queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )?;

        let [width, height] = self.swapchain.image_extent().map(|x| x as f32);
        let mut projection =
            glam::Mat4::perspective_rh(f32::to_radians(45.0), width / height, 0.1, 100.0);
        projection.as_mut()[1 * 4 + 1] *= -1.0;

        builder
            .begin_render_pass(render_pass_begin_info, subpass_begin_info)?
            .bind_pipeline_graphics(Arc::clone(pipeline))?
            .push_constants(
                Arc::clone(layout),
                16 * size_of::<f32>() as u32,
                camera.get_view(),
            )?
            .push_constants(
                Arc::clone(layout),
                2 * 16 * size_of::<f32>() as u32,
                projection,
            )?
            .set_viewport(
                0,
                [Viewport {
                    offset: [0.0, 0.0],
                    extent: self.swapchain.image_extent().map(|x| x as f32),
                    depth_range: 0.0..=1.0,
                }]
                .into_iter()
                .collect(),
            )?
            .set_scissor(
                0,
                [Scissor {
                    offset: [0, 0],
                    extent: self.swapchain.image_extent(),
                }]
                .into_iter()
                .collect(),
            )?;

        for (_, mesh_component) in scene.components::<MeshComponent>().unwrap() {
            let vertex_buffer = mesh_component.mesh.vectex_buffer();
            let index_buffer = mesh_component.mesh.index_buffer();

            builder
                .bind_vertex_buffers(0, vertex_buffer.clone())?
                .bind_index_buffer(index_buffer.clone())?
                .push_constants(Arc::clone(layout), 0, mesh_component.model.transform())?
                .draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0)?;
        }

        builder.end_render_pass(subpass_end_info)?;

        let command_buffer = builder.build()?;

        Ok(command_buffer)
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
        vulkan_context: &Arc<VulkanContext>,
        window: &Arc<Window>,
    ) -> Result<(Arc<Swapchain>, Vec<Arc<Image>>)> {
        let device = vulkan_context.device();
        let physical_device = device.physical_device();

        let surface_info = SurfaceInfo {
            full_screen_exclusive: FullScreenExclusive::Default,
            ..Default::default()
        };

        let surface =
            Surface::from_window(Arc::clone(vulkan_context.instance()), Arc::clone(window))?;

        let surface_capabilities =
            physical_device.surface_capabilities(surface.as_ref(), surface_info.clone())?;

        let available_formats =
            physical_device.surface_formats(surface.as_ref(), surface_info.clone())?;

        let (format, color_space) = Self::choose_swapchain_format(available_formats);
        let extent = Self::choose_swapchain_extent(window, &surface_capabilities);

        let sharing = Sharing::Exclusive;

        let available_present_modes = physical_device
            .surface_present_modes(surface.as_ref(), surface_info)?
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

        let (swapchain, swapchain_images) =
            Swapchain::new(device.clone(), surface.clone(), swapchain_info)?;

        Ok((swapchain, swapchain_images))
    }

    fn create_swapchain_image_views(
        swapchain: &Arc<Swapchain>,
        swapchain_images: &Vec<Arc<Image>>,
    ) -> Result<Vec<Arc<ImageView>>> {
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

            image_views.push(ImageView::new(image.clone(), view_info)?);
        }

        Ok(image_views)
    }

    fn create_framebuffers(
        render_pass: &Arc<RenderPass>,
        swapchain: &Arc<Swapchain>,
        image_views: &Vec<Arc<ImageView>>,
        depth_image_view: &Arc<ImageView>,
    ) -> Result<Vec<Arc<Framebuffer>>> {
        let mut framebuffers = Vec::new();

        for image_view in image_views.iter() {
            let framebuffer_info = FramebufferCreateInfo {
                attachments: vec![Arc::clone(image_view), Arc::clone(depth_image_view)],
                extent: swapchain.image_extent(),
                layers: 1,
                ..Default::default()
            };

            framebuffers.push(Framebuffer::new(render_pass.clone(), framebuffer_info)?);
        }

        Ok(framebuffers)
    }

    fn create_depth_image(
        vulkan_context: &Arc<VulkanContext>,
        image_extent: [u32; 2],
    ) -> Result<(Arc<Image>, Arc<ImageView>)> {
        let allocator = Arc::clone(vulkan_context.standard_memory_allocator());

        let depth_image = Image::new(
            allocator,
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::D32_SFLOAT,
                view_formats: vec![Format::D32_SFLOAT],
                extent: [image_extent[0], image_extent[1], 1],
                usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                sharing: Sharing::Exclusive,
                initial_layout: ImageLayout::Undefined,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                allocate_preference: MemoryAllocatePreference::AlwaysAllocate,
                ..Default::default()
            },
        )?;

        let depth_image_view = ImageView::new(
            Arc::clone(&depth_image),
            ImageViewCreateInfo {
                view_type: ImageViewType::Dim2d,
                format: depth_image.format(),
                component_mapping: ComponentMapping::identity(),
                subresource_range: ImageSubresourceRange {
                    aspects: ImageAspects::DEPTH,
                    mip_levels: 0..1,
                    array_layers: 0..1,
                },
                usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                ..Default::default()
            },
        )?;

        Ok((depth_image, depth_image_view))
    }

    fn create_render_pass(
        device: &Arc<Device>,
        swapchain: &Arc<Swapchain>,
        depth_stencil_image: &Arc<Image>,
    ) -> Arc<RenderPass> {
        let color_attachment = AttachmentDescription {
            format: swapchain.image_format(),
            samples: SampleCount::Sample1,
            load_op: AttachmentLoadOp::Clear,
            store_op: AttachmentStoreOp::Store,
            initial_layout: ImageLayout::Undefined,
            final_layout: ImageLayout::PresentSrc,
            ..Default::default()
        };

        let color_attachment_ref = AttachmentReference {
            attachment: 0,
            layout: ImageLayout::ColorAttachmentOptimal,
            ..Default::default()
        };

        let depth_attachment = AttachmentDescription {
            format: depth_stencil_image.format(),
            samples: SampleCount::Sample1,
            load_op: AttachmentLoadOp::Clear,
            store_op: AttachmentStoreOp::DontCare,
            initial_layout: ImageLayout::Undefined,
            final_layout: ImageLayout::DepthStencilAttachmentOptimal,
            ..Default::default()
        };

        let depth_attachment_ref = AttachmentReference {
            attachment: 1,
            layout: ImageLayout::DepthStencilAttachmentOptimal,
            ..Default::default()
        };

        let subpass = SubpassDescription {
            view_mask: 0,
            color_attachments: vec![Some(color_attachment_ref)],
            depth_stencil_attachment: Some(depth_attachment_ref),
            ..Default::default()
        };

        let attachments = vec![color_attachment, depth_attachment];
        let subpasses = vec![subpass];
        let dependencies = vec![];

        let render_pass_info = RenderPassCreateInfo {
            attachments,
            subpasses,
            dependencies,
            ..Default::default()
        };

        RenderPass::new(device.clone(), render_pass_info).expect("Failed to create render pass")
    }

    pub(crate) fn resize(&mut self, new_size: PhysicalSize<u32>) -> Result<()> {
        let (new_swapchain, new_swapchain_images) =
            self.swapchain.recreate(SwapchainCreateInfo {
                image_extent: [new_size.width, new_size.height],
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                ..self.swapchain.create_info()
            })?;

        let new_swapchain_image_views =
            Self::create_swapchain_image_views(&new_swapchain, &new_swapchain_images)?;

        let (new_depth_image, new_depth_image_view) =
            Self::create_depth_image(&self.vulkan_context, new_swapchain.image_extent())?;

        let new_framebuffers = Self::create_framebuffers(
            &self.render_pass,
            &new_swapchain,
            &new_swapchain_image_views,
            &new_depth_image_view,
        )?;

        self.swapchain = new_swapchain;
        self._swapchain_images = new_swapchain_images;
        self._swapchain_image_views = new_swapchain_image_views;

        self.depth_image = new_depth_image;
        self.depth_image_view = new_depth_image_view;

        self.framebuffers = new_framebuffers;

        Ok(())
    }
}
