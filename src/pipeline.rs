use std::collections::HashMap;
use std::path::Path;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::str;
use crate::binding;
use std::borrow::Cow;
use anyhow::*;

pub struct RenderPipeline{
    pub pipeline: wgpu::RenderPipeline,
    pub sets: HashMap<String, usize>,
}

pub struct PipelineLayout{
    pub layout: wgpu::PipelineLayout,
    pub sets: HashMap<String, usize>,
}

pub struct PipelineLayoutBuilder<'l>{
    bind_group_layouts: Vec<&'l binding::BindGroupLayoutWithDesc>,
    push_constant_ranges: Vec<wgpu::PushConstantRange>,
    sets: HashMap<String, usize>,
    index: usize,
}

impl<'l> PipelineLayoutBuilder<'l>{
    pub fn new() -> Self{
        Self{
            bind_group_layouts: Vec::new(),
            push_constant_ranges: Vec::new(),
            sets: HashMap::new(),
            index: 0,
        }
    }

    pub fn push_named(mut self, name: &str, bind_group_layout: &'l binding::BindGroupLayoutWithDesc) -> Self{
        if let Some(index) = self.sets.get(name){
            self.bind_group_layouts.remove(*index);
            self.bind_group_layouts.insert(*index, bind_group_layout);
        }
        else{
            self.sets.insert(name.to_string(), self.index);
            self.index += 1;
            self.bind_group_layouts.push(bind_group_layout);
        }
        self
    }

    pub fn push_push_constant_ranges(mut self, push_constant_ranges: wgpu::PushConstantRange) -> Self{
        self.push_constant_ranges.push(push_constant_ranges);
        self
    }

    pub fn create(self, device: &wgpu::Device, label: Option<&str>) -> PipelineLayout{

        let mut bind_group_layouts = Vec::with_capacity(self.bind_group_layouts.len());
        for bind_group_layout_desc in self.bind_group_layouts{
            bind_group_layouts.push(&bind_group_layout_desc.layout);
        }

        PipelineLayout{
            layout: device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
                label,
                bind_group_layouts: &bind_group_layouts,
                push_constant_ranges: &self.push_constant_ranges,
            }),
            sets: self.sets,
        }
    }
}

pub struct RenderPass<'rp>{
    pub pass: wgpu::RenderPass<'rp>,
    pub pipeline: Option<&'rp RenderPipeline>,
}

impl<'rp> RenderPass<'rp>{
    #[inline]
    pub fn set_pipeline(&mut self, pipeline: &'rp RenderPipeline){
        self.pipeline = Some(pipeline);
        self.pass.set_pipeline(&pipeline.pipeline);
    }

    #[inline]
    pub fn set_bind_group(&mut self, index: u32, bind_group: &'rp wgpu::BindGroup, offsets: &'rp [wgpu::DynamicOffset]){
        self.pass.set_bind_group(index, bind_group, offsets);
    }

    #[inline]
    pub fn set_bind_group_named(&mut self, name: &str, bind_group: &'rp wgpu::BindGroup, offsets: &'rp [wgpu::DynamicOffset]){
        if let Some(pipeline) = self.pipeline{
            self.pass.set_bind_group(pipeline.sets[name] as u32, bind_group, offsets);
        }
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
    pub fn begin(self, encoder: &'rp mut wgpu::CommandEncoder, label: Option<&'rp str>) -> RenderPass<'rp>{
        RenderPass{
            pass: encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
                label,
                color_attachments: &self.color_attachments,
                depth_stencil_attachment: None,
            }),
            pipeline: None,
        }
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
