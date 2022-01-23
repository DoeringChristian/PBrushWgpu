use std::path::Path;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::str;
use std::borrow::Cow;
use anyhow::*;

pub struct RenderPipelineLayoutBuilder<'l>{
    bind_group_layouts: Vec<&'l wgpu::BindGroupLayout>,
    push_constant_ranges: Vec<wgpu::PushConstantRange>,
}

impl<'l> RenderPipelineLayoutBuilder<'l>{
    fn new() -> Self{
        Self{
            bind_group_layouts: Vec::new(),
            push_constant_ranges: Vec::new(),
        }
    }

    fn push_bind_group_layout(mut self, bind_group_layout: &'l wgpu::BindGroupLayout) -> Self{
        self.bind_group_layouts.push(bind_group_layout);
        self
    }

    fn push_bind_group_layouts(mut self, bind_group_layouts: &[&'l wgpu::BindGroupLayout]) -> Self{
        for bind_group_layout in bind_group_layouts{
            self.bind_group_layouts.push(&bind_group_layout);
        }
        self
    }

    fn create(self, device: &wgpu::Device, label: Option<&str>) -> wgpu::PipelineLayout{
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
            label,
            bind_group_layouts: &self.bind_group_layouts,
            push_constant_ranges: &self.push_constant_ranges,
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
