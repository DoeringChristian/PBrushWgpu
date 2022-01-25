use std::collections::HashMap;
use std::path::Path;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::str;
use std::sync::Arc;
use crate::binding;
use std::borrow::Cow;
use anyhow::*;


///
/// Layout of the VertexState of a Pipeline.
/// It describes the buffer layouts as well as the names used when setting by name in the 
/// RenderPassPipeline process.
///
pub struct VertexStateLayout<'vs>{
    pub vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'vs>>,
    /// used to save names and corresponding indices.
    pub vertex_buffer_names: Arc<HashMap<String, usize>>,
    pub entry_point: &'vs str,
}

pub struct VertexStateLayoutBuilder<'vsb>{
    vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'vsb>>,
    vertex_buffer_names: HashMap<String, usize>,
    entry_point: &'vsb str,
    //module: &'vsb wgpu::ShaderModule,
    index: usize,
}

impl <'vsb> VertexStateLayoutBuilder<'vsb>{
    pub fn new() -> Self{
        Self{
            vertex_buffer_layouts: Vec::new(),
            vertex_buffer_names: HashMap::new(),
            entry_point: "vs_main",
            index: 0,
        }
    }

    pub fn set_entry_point(mut self, entry_point: &'vsb str) -> Self{
        self.entry_point = entry_point;
        self
    }

    pub fn push_named(mut self, name: &str, vertex_buffer_layout: wgpu::VertexBufferLayout<'vsb>) -> Self{
        if let Some(index) = self.vertex_buffer_names.get(name){
            self.vertex_buffer_layouts.remove(*index);
            self.vertex_buffer_layouts.insert(*index, vertex_buffer_layout);
        }
        else{
            self.vertex_buffer_names.insert(name.to_string(), self.index);
            self.index += 1;
            self.vertex_buffer_layouts.push(vertex_buffer_layout);
        }
        self
    }

    pub fn build(self) -> VertexStateLayout<'vsb>{
        VertexStateLayout{
            vertex_buffer_names: Arc::new(self.vertex_buffer_names),
            vertex_buffer_layouts: self.vertex_buffer_layouts,
            entry_point: self.entry_point,
        }
    }
}




pub struct RenderPipeline{
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_names: Arc<HashMap<String, usize>>,
    pub vertex_buffer_names: Arc<HashMap<String, usize>>,
}

pub struct PipelineLayout{
    pub layout: wgpu::PipelineLayout,
    pub names: Arc<HashMap<String, usize>>,
}

// TODO: put bind_group_names in Arc
pub struct PipelineLayoutBuilder<'l>{
    bind_group_layouts: Vec<&'l binding::BindGroupLayoutWithDesc>,
    push_constant_ranges: Vec<wgpu::PushConstantRange>,
    bind_group_names: HashMap<String, usize>,
    index: usize,
}

impl<'l> PipelineLayoutBuilder<'l>{
    pub fn new() -> Self{
        Self{
            bind_group_layouts: Vec::new(),
            push_constant_ranges: Vec::new(),
            bind_group_names: HashMap::new(),
            index: 0,
        }
    }

    pub fn push_named(mut self, name: &str, bind_group_layout: &'l binding::BindGroupLayoutWithDesc) -> Self{
        if let Some(index) = self.bind_group_names.get(name){
            self.bind_group_layouts.remove(*index);
            self.bind_group_layouts.insert(*index, bind_group_layout);
        }
        else{
            self.bind_group_names.insert(name.to_string(), self.index);
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
            names: Arc::new(self.bind_group_names),
        }
    }
}

///
/// A Render Pass with a Pipeline.
///
/// used to reference that Pipeline so one is able to set the bind groups and vertex buffers by
/// name.
/// 
///

pub struct RenderPassPointer<'rp, 'rpp>{
    pub render_pass: &'rpp RenderPass<'rp>,
}

pub struct RenderPassPipeline<'rp>{
    pub render_pass: RenderPass<'rp>,
    pub pipeline: &'rp RenderPipeline,
}

impl<'rp> RenderPassPipeline<'rp>{
    pub fn set_bind_group_named(&mut self, name: &str, bind_group: &'rp wgpu::BindGroup, offsets: &'rp [wgpu::DynamicOffset]){
        self.render_pass.render_pass.set_bind_group(
            self.pipeline.bind_group_names[name] as u32, 
            bind_group, offsets
        );
    }

    pub fn set_vertex_buffer(&mut self, name: &str, buffer_slice: wgpu::BufferSlice<'rp>){
        self.render_pass.render_pass.set_vertex_buffer(
            self.pipeline.vertex_buffer_names[name] as u32, 
            buffer_slice
        );
    }

    pub fn set_index_buffer(&mut self, buffer_slice: wgpu::BufferSlice<'rp>, format: wgpu::IndexFormat){
        self.render_pass.render_pass.set_index_buffer(buffer_slice, format);
    }
}

pub struct RenderPass<'rp>{
    pub render_pass: wgpu::RenderPass<'rp>,
}

impl<'rp> RenderPass<'rp>{
    pub fn set_pipeline(mut self, pipeline: &'rp RenderPipeline) -> RenderPassPipeline<'rp>{
        self.render_pass.set_pipeline(&pipeline.pipeline);
        RenderPassPipeline{
            render_pass: self,
            pipeline,
        }
    }

    /*
    #[inline]
    pub fn set_bind_group(&mut self, index: u32, bind_group: &'rp wgpu::BindGroup, offsets: &'rp [wgpu::DynamicOffset]){
        self.render_pass.set_bind_group(index, bind_group, offsets);
    }
    */
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
            render_pass: encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
                label,
                color_attachments: &self.color_attachments,
                depth_stencil_attachment: None,
            }),
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
