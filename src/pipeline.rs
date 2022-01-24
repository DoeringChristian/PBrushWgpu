use std::path::Path;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::str;
use crate::binding;
use std::borrow::Cow;
use anyhow::*;

pub struct RenderPipelineLayoutBuilder<'l>{
    bind_group_layouts_desc: Vec<&'l binding::BindGroupLayoutWithDesc>,
    bind_group_layouts: Vec<&'l wgpu::BindGroupLayout>,
    push_constant_ranges: Vec<wgpu::PushConstantRange>,
}

impl<'l> RenderPipelineLayoutBuilder<'l>{
    pub fn new() -> Self{
        Self{
            bind_group_layouts_desc: Vec::new(),
            bind_group_layouts: Vec::new(),
            push_constant_ranges: Vec::new(),
        }
    }

    pub fn push_bind_group_layout(mut self, bind_group_layout: &'l binding::BindGroupLayoutWithDesc) -> Self{
        self.bind_group_layouts_desc.push(bind_group_layout);
        self.bind_group_layouts.push(&bind_group_layout.layout);
        self
    }

    pub fn push_bind_group_layouts(mut self, bind_group_layouts: &[&'l binding::BindGroupLayoutWithDesc]) -> Self{
        for bind_group_layout in bind_group_layouts{
            self.bind_group_layouts_desc.push(&bind_group_layout);
            self.bind_group_layouts.push(&bind_group_layout.layout);
        }
        self
    }

    pub fn push_push_constant_ranges(mut self, push_constant_ranges: wgpu::PushConstantRange) -> Self{
        self.push_constant_ranges.push(push_constant_ranges);
        self
    }

    pub fn create(self, device: &wgpu::Device, label: Option<&str>) -> wgpu::PipelineLayout{
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
            label,
            bind_group_layouts: &self.bind_group_layouts,
            push_constant_ranges: &self.push_constant_ranges,
        })
    }
}

pub struct RenderPassBuilder<'rp>{
    color_attachments: Vec<wgpu::RenderPassColorAttachment<'rp>>,
}

impl<'rp> RenderPassBuilder<'rp>{
    pub fn new() -> Self{
        Self{
            color_attachments: Vec::new(),
        }
    }

    pub fn push_color_attachment(mut self, color_attachment: wgpu::RenderPassColorAttachment<'rp>) -> Self{
        self.color_attachments.push(color_attachment);
        self
    }

    // TODO: add depth_stencil_attachment
    pub fn begin(self, encoder: &'rp mut wgpu::CommandEncoder, label: Option<&'rp str>) -> wgpu::RenderPass<'rp>{
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
            label,
            color_attachments: &self.color_attachments,
            depth_stencil_attachment: None,
        })
    }
}


fn shader_load(device: &wgpu::Device, path: &str, label: Option<&str>) -> Result<wgpu::ShaderModule>{
    let mut f = File::open(path)?;
    let metadata = fs::metadata(path)?;
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer)?;
    let src = str::from_utf8(&buffer)?;


    let extension = Path::new(path).extension().ok_or(anyhow!("No extension"))?;

    let source = match extension.to_str().ok_or(anyhow!("string conversion"))?{
        //"glsl" => wgpu::ShaderSource::Glsl(src),
        "wgsl" => wgpu::ShaderSource::Wgsl(Cow::from(src)),
        _ => return Err(anyhow!("Unknown Extension")),
    };

    Ok(device.create_shader_module(&wgpu::ShaderModuleDescriptor{
        label,
        source,
    }))
}

struct RenderPipelineBuilder{
    shader: wgpu::ShaderModule,
}

// TODO:
// Counting RenderPass
