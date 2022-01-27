
use crate::{binding, pipeline};


use anyhow::*;
pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, pipeline_layout: &pipeline::PipelineLayout, vertex_stage: &pipeline::VertexState, fragment_stage: &pipeline::FragmentState) -> Result<pipeline::RenderPipeline>{

    /*
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
        label: Some("Pipeline layout"),
        bind_group_layouts,
        push_constant_ranges: &[],
    });
    */

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout.layout),
        vertex: wgpu::VertexState{
            module: &vertex_stage.vertex_shader,
            entry_point: vertex_stage.entry_point,
            buffers: &vertex_stage.vertex_buffer_layouts,
        },
        fragment: Some(wgpu::FragmentState{
            module: &fragment_stage.shader,
            entry_point: fragment_stage.entry_point,
            targets: &[wgpu::ColorTargetState{
                format,
                blend: Some(wgpu::BlendState{
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::all(),
            }],
        }),
        primitive: wgpu::PrimitiveState{
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            //cull_mode: Some(wgpu::Face::Back),
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState{
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    Ok(
        pipeline::RenderPipeline{
            pipeline: render_pipeline,
            bind_group_names: pipeline_layout.names.clone(),
            vertex_buffer_names: vertex_stage.vertex_buffer_names.clone(),
        }
    )
}
