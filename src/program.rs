use anyhow::*;
use wgpu::util::DeviceExt;

pub struct Program{
    pub render_pipeline: wgpu::RenderPipeline,
}

impl Program{
    pub fn new(device: &wgpu::Device, src: &str, format: wgpu::TextureFormat, bind_group_layouts: &[&wgpu::BindGroupLayout], vertex_buffer_layouts: &[wgpu::VertexBufferLayout]) -> Result<Self>{

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor{
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(src.into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
            label: Some("Pipeline layout"),
            bind_group_layouts,
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
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
                cull_mode: Some(wgpu::Face::Back),
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

        Ok(Self{
            render_pipeline,
        })
    }
}
