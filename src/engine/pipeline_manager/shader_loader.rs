use std::{mem::size_of, sync::Arc};

use glam::Mat4;
use vulkano::{
    descriptor_set::layout::DescriptorSetLayout,
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
use vulkano_shaders;

use anyhow::Result;

use super::VulkanPipeline;
use crate::engine::mesh::Vertex as MyVertex;

pub fn load_depth(device: &Arc<Device>, render_pass: &Arc<RenderPass>) -> Result<VulkanPipeline> {
    vulkano_shaders::shader! {
        shaders: {
            vertex: {
                ty: "vertex",
                path: "shaders/debug/depth.vert"
            },
            fragment: {
                ty: "fragment",
                path: "shaders/debug/depth.frag"
            }
        }
    }

    let vertex_shader = load_vertex(Arc::clone(device))?
        .entry_point("main")
        .unwrap();
    let fragment_shader = load_fragment(Arc::clone(device))?
        .entry_point("main")
        .unwrap();

    let vertex_input_state =
        MyVertex::per_vertex().definition(&vertex_shader.info().input_interface)?;

    let pipeline_layout = {
        let layout_info = PipelineLayoutCreateInfo {
            flags: PipelineLayoutCreateFlags::empty(),
            push_constant_ranges: vec![PushConstantRange {
                stages: ShaderStages::VERTEX,
                offset: 0,
                size: 3 * size_of::<Mat4>() as u32,
            }],
            ..Default::default()
        };

        PipelineLayout::new(Arc::clone(device), layout_info)?
    };

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

pub fn load_normal(device: &Arc<Device>, render_pass: &Arc<RenderPass>) -> Result<VulkanPipeline> {
    vulkano_shaders::shader! {
        shaders: {
            vertex: {
                ty: "vertex",
                path: "shaders/debug/normal.vert"
            },
            fragment: {
                ty: "fragment",
                path: "shaders/debug/normal.frag"
            }
        }
    }

    let vertex_shader = load_vertex(Arc::clone(device))?
        .entry_point("main")
        .unwrap();
    let fragment_shader = load_fragment(Arc::clone(device))?
        .entry_point("main")
        .unwrap();

    let vertex_input_state =
        MyVertex::per_vertex().definition(&vertex_shader.info().input_interface)?;

    let pipeline_layout = {
        let layout_info = PipelineLayoutCreateInfo {
            flags: PipelineLayoutCreateFlags::empty(),
            push_constant_ranges: vec![PushConstantRange {
                stages: ShaderStages::VERTEX,
                offset: 0,
                size: 3 * size_of::<Mat4>() as u32,
            }],
            ..Default::default()
        };

        PipelineLayout::new(Arc::clone(device), layout_info)?
    };

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

pub fn load_mesh_view(
    device: &Arc<Device>,
    render_pass: &Arc<RenderPass>,
) -> Result<VulkanPipeline> {
    vulkano_shaders::shader! {
        shaders: {
            vertex: {
                ty: "vertex",
                path: "shaders/debug/mesh_view.vert"
            },
            fragment: {
                ty: "fragment",
                path: "shaders/debug/mesh_view.frag"
            }
        }
    }

    let vertex_shader = load_vertex(Arc::clone(device))?
        .entry_point("main")
        .unwrap();
    let fragment_shader = load_fragment(Arc::clone(device))?
        .entry_point("main")
        .unwrap();

    let vertex_input_state =
        MyVertex::per_vertex().definition(&vertex_shader.info().input_interface)?;

    let pipeline_layout = {
        let layout_info = PipelineLayoutCreateInfo {
            flags: PipelineLayoutCreateFlags::empty(),
            push_constant_ranges: vec![PushConstantRange {
                stages: ShaderStages::VERTEX,
                offset: 0,
                size: 3 * size_of::<Mat4>() as u32,
            }],
            ..Default::default()
        };

        PipelineLayout::new(Arc::clone(device), layout_info)?
    };

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

pub fn load_material_simple(
    device: &Arc<Device>,
    render_pass: &Arc<RenderPass>,
    material_set_layout: Arc<DescriptorSetLayout>,
) -> Result<VulkanPipeline> {
    vulkano_shaders::shader! {
        shaders: {
            vertex: {
                ty: "vertex",
                path: "shaders/material/simple.vert"
            },
            fragment: {
                ty: "fragment",
                path: "shaders/material/simple.frag"
            }
        }
    }

    let vertex_shader = load_vertex(Arc::clone(device))?
        .entry_point("main")
        .unwrap();
    let fragment_shader = load_fragment(Arc::clone(device))?
        .entry_point("main")
        .unwrap();

    let vertex_input_state =
        MyVertex::per_vertex().definition(&vertex_shader.info().input_interface)?;

    let pipeline_layout = {
        let layout_info = PipelineLayoutCreateInfo {
            flags: PipelineLayoutCreateFlags::empty(),
            set_layouts: vec![material_set_layout],
            push_constant_ranges: vec![PushConstantRange {
                stages: ShaderStages::VERTEX,
                offset: 0,
                size: 3 * size_of::<Mat4>() as u32,
            }],
            ..Default::default()
        };

        PipelineLayout::new(Arc::clone(device), layout_info)?
    };

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
