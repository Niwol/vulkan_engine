use std::{mem::size_of, sync::Arc};

use vulkano::{
    device::Device,
    pipeline::{
        graphics::{
            color_blend::{
                ColorBlendAttachmentState, ColorBlendState, ColorBlendStateFlags, ColorComponents,
            },
            depth_stencil::{CompareOp, DepthState, DepthStencilState},
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::{
                CullMode, FrontFace, LineRasterizationMode, PolygonMode, RasterizationState,
            },
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Scissor, Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::{PipelineLayoutCreateFlags, PipelineLayoutCreateInfo, PushConstantRange},
        DynamicState, GraphicsPipeline, PipelineCreateFlags, PipelineLayout,
        PipelineShaderStageCreateInfo,
    },
    render_pass::{RenderPass, Subpass},
    shader::ShaderStages,
};

use anyhow::Result;

use crate::engine::mesh::Vertex as MyVertex;
use crate::vulkan_context::VulkanContext;

mod shader_loader;

pub struct VulkanPipeline {
    pub pipeline: Arc<GraphicsPipeline>,
    pub layout: Arc<PipelineLayout>,
}

pub struct PipelineManager {
    normal_pipeline: VulkanPipeline,
    depth_pipeline: VulkanPipeline,
    mesh_view_pipeine: VulkanPipeline,
}

impl PipelineManager {
    pub fn new(vulkan_context: &Arc<VulkanContext>, render_pass: &Arc<RenderPass>) -> Result<Self> {
        let device = vulkan_context.device();

        let normal_pipeline = create_normal_pipeline(device, render_pass)?;
        let depth_pipeline = create_depth_pipeline(device, render_pass)?;
        let mesh_view_pipeine = create_mesh_view_pipeline(device, render_pass)?;

        Ok(Self {
            normal_pipeline,
            depth_pipeline,
            mesh_view_pipeine,
        })
    }

    pub fn normal_pipeline(&self) -> &VulkanPipeline {
        &self.normal_pipeline
    }

    pub fn depth_pipeline(&self) -> &VulkanPipeline {
        &self.depth_pipeline
    }

    pub fn mesh_view_pipeine(&self) -> &VulkanPipeline {
        &self.mesh_view_pipeine
    }
}

fn create_mesh_view_pipeline(
    device: &Arc<Device>,
    render_pass: &Arc<RenderPass>,
) -> Result<VulkanPipeline> {
    let vertex_shader = shader_loader::load_mesh_view_vert(Arc::clone(device))?
        .entry_point("main")
        .unwrap();
    let fragment_shader = shader_loader::load_mesh_view_frag(Arc::clone(device))?
        .entry_point("main")
        .unwrap();

    let vertex_input_state =
        MyVertex::per_vertex().definition(&vertex_shader.info().input_interface)?;

    let pipeline_layout = create_pipeline_layout(device)?;

    let pipeline_info = GraphicsPipelineCreateInfo {
        flags: PipelineCreateFlags::empty(),
        stages: [
            PipelineShaderStageCreateInfo::new(vertex_shader),
            PipelineShaderStageCreateInfo::new(fragment_shader),
        ]
        .into_iter()
        .collect(),
        vertex_input_state: Some(vertex_input_state),
        input_assembly_state: Some(InputAssemblyState {
            topology: PrimitiveTopology::TriangleList,
            primitive_restart_enable: false,
            ..Default::default()
        }),
        tessellation_state: None,
        viewport_state: Some(ViewportState {
            viewports: [Viewport {
                offset: [0.0, 0.0],
                extent: [800.0, 600.0],
                ..Default::default()
            }]
            .into_iter()
            .collect(),
            scissors: [Scissor {
                offset: [0, 0],
                extent: [800, 600],
            }]
            .into_iter()
            .collect(),
            ..Default::default()
        }),
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
        depth_stencil_state: Some(DepthStencilState {
            depth: Some(DepthState {
                write_enable: true,
                compare_op: CompareOp::Less,
            }),
            ..Default::default()
        }),
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

        dynamic_state: [DynamicState::Viewport, DynamicState::Scissor]
            .into_iter()
            .collect(),

        ..GraphicsPipelineCreateInfo::layout(pipeline_layout.clone())
    };

    let pipeline = GraphicsPipeline::new(device.clone(), None, pipeline_info)?;

    Ok(VulkanPipeline {
        pipeline,
        layout: pipeline_layout,
    })
}

fn create_depth_pipeline(
    device: &Arc<Device>,
    render_pass: &Arc<RenderPass>,
) -> Result<VulkanPipeline> {
    let vertex_shader = shader_loader::load_depth_vert(Arc::clone(device))?
        .entry_point("main")
        .unwrap();
    let fragment_shader = shader_loader::load_depth_frag(Arc::clone(device))?
        .entry_point("main")
        .unwrap();

    let vertex_input_state =
        MyVertex::per_vertex().definition(&vertex_shader.info().input_interface)?;

    let pipeline_layout = create_pipeline_layout(device)?;

    let pipeline_info = GraphicsPipelineCreateInfo {
        flags: PipelineCreateFlags::empty(),
        stages: [
            PipelineShaderStageCreateInfo::new(vertex_shader),
            PipelineShaderStageCreateInfo::new(fragment_shader),
        ]
        .into_iter()
        .collect(),
        vertex_input_state: Some(vertex_input_state),
        input_assembly_state: Some(InputAssemblyState {
            topology: PrimitiveTopology::TriangleList,
            primitive_restart_enable: false,
            ..Default::default()
        }),
        tessellation_state: None,
        viewport_state: Some(ViewportState {
            viewports: [Viewport {
                offset: [0.0, 0.0],
                extent: [800.0, 600.0],
                ..Default::default()
            }]
            .into_iter()
            .collect(),
            scissors: [Scissor {
                offset: [0, 0],
                extent: [800, 600],
            }]
            .into_iter()
            .collect(),
            ..Default::default()
        }),
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
        depth_stencil_state: Some(DepthStencilState {
            depth: Some(DepthState {
                write_enable: true,
                compare_op: CompareOp::Less,
            }),
            ..Default::default()
        }),
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

        dynamic_state: [DynamicState::Viewport, DynamicState::Scissor]
            .into_iter()
            .collect(),

        ..GraphicsPipelineCreateInfo::layout(pipeline_layout.clone())
    };

    let pipeline = GraphicsPipeline::new(device.clone(), None, pipeline_info)?;

    Ok(VulkanPipeline {
        pipeline,
        layout: pipeline_layout,
    })
}

fn create_normal_pipeline(
    device: &Arc<Device>,
    render_pass: &Arc<RenderPass>,
) -> Result<VulkanPipeline> {
    let vertex_shader = shader_loader::load_normal_vert(Arc::clone(device))?
        .entry_point("main")
        .unwrap();
    let fragment_shader = shader_loader::load_normal_frag(Arc::clone(device))?
        .entry_point("main")
        .unwrap();

    let vertex_input_state =
        MyVertex::per_vertex().definition(&vertex_shader.info().input_interface)?;

    let pipeline_layout = create_pipeline_layout(device)?;

    let pipeline_info = GraphicsPipelineCreateInfo {
        flags: PipelineCreateFlags::empty(),
        stages: [
            PipelineShaderStageCreateInfo::new(vertex_shader),
            PipelineShaderStageCreateInfo::new(fragment_shader),
        ]
        .into_iter()
        .collect(),
        vertex_input_state: Some(vertex_input_state),
        input_assembly_state: Some(InputAssemblyState {
            topology: PrimitiveTopology::TriangleList,
            primitive_restart_enable: false,
            ..Default::default()
        }),
        tessellation_state: None,
        viewport_state: Some(ViewportState {
            viewports: [Viewport {
                offset: [0.0, 0.0],
                extent: [800.0, 600.0],
                ..Default::default()
            }]
            .into_iter()
            .collect(),
            scissors: [Scissor {
                offset: [0, 0],
                extent: [800, 600],
            }]
            .into_iter()
            .collect(),
            ..Default::default()
        }),
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
        depth_stencil_state: Some(DepthStencilState {
            depth: Some(DepthState {
                write_enable: true,
                compare_op: CompareOp::Less,
            }),
            ..Default::default()
        }),
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

        dynamic_state: [DynamicState::Viewport, DynamicState::Scissor]
            .into_iter()
            .collect(),

        ..GraphicsPipelineCreateInfo::layout(pipeline_layout.clone())
    };

    let pipeline = GraphicsPipeline::new(device.clone(), None, pipeline_info)?;

    Ok(VulkanPipeline {
        pipeline,
        layout: pipeline_layout,
    })
}

fn create_pipeline(create_info: GraphicsPipelineCreateInfo) -> VulkanPipeline {
    todo!()
}

fn create_pipeline_layout(device: &Arc<Device>) -> Result<Arc<PipelineLayout>> {
    let layout_info = PipelineLayoutCreateInfo {
        flags: PipelineLayoutCreateFlags::empty(),
        set_layouts: Vec::new(),
        push_constant_ranges: vec![PushConstantRange {
            stages: ShaderStages::VERTEX,
            offset: 0,
            size: 3 * 16 * size_of::<f32>() as u32,
        }],
        ..Default::default()
    };

    let pipeline_layout = PipelineLayout::new(device.clone(), layout_info)?;

    Ok(pipeline_layout)
}
