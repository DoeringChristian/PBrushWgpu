
use crate::{binding, pipeline};


use anyhow::*;
pub fn new(device: &wgpu::Device, src: &str, format: wgpu::TextureFormat, pipeline_layout: &pipeline::PipelineLayout, vertex_buffer_layouts: &[wgpu::VertexBufferLayout]) -> Result<pipeline::RenderPipeline>{

    let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor{
        label: Some("shader"),
        source: wgpu::ShaderSource::Wgsl(src.into()),
    });

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
            module: &shader,
            entry_point: "vs_main",
            buffers: vertex_buffer_layouts,
        },
        fragment: Some(wgpu::FragmentState{
            module: &shader,
            entry_point: "fs_main",
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
            sets: pipeline_layout.sets.clone(),
        }
    )
}
