use std::collections::HashMap;
use std::ops::Deref;
use std::path::Path;
use std::{fs, ops};
use std::fs::File;
use std::io::Read;
use std::str;
use std::sync::Arc;
use crate::binding;
use std::borrow::Cow;
use anyhow::*;
use core::ops::Range;
use naga;

pub struct FragmentState<'fs>{
    pub color_target_states: Vec<wgpu::ColorTargetState>,
    pub entry_point: &'fs str,
    pub shader: &'fs wgpu::ShaderModule,
}

pub struct FragmentStateBuilder<'fsb>{
    pub color_target_states: Vec<wgpu::ColorTargetState>,
    shader: &'fsb wgpu::ShaderModule,
    entry_point: &'fsb str,
}

impl <'fsb> FragmentStateBuilder<'fsb>{
    pub fn new(shader: &'fsb wgpu::ShaderModule) -> Self{
        Self{
            color_target_states: Vec::new(),
            shader,
            entry_point: "fs_main",
        }
    }

    // TODO: create shader in build
    /*
    pub fn new_from_src_glsl<'s>(device: &wgpu::Device, src: &'s str) -> Self{
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor{
            label: None,
            source: wgpu::ShaderSource::Glsl{
                shader: Cow::from(src),
                stage: naga::ShaderStage::Fragment,
                defines: naga::FastHashMap::default()
            }
        });

        Self{
            color_target_states: Vec::new(),
            entry_point: "fs_main",
            shader: &shader,
        }
    }
    */
    
    pub fn set_entry_point(mut self, entry_point: &'fsb str) -> Self{
        self.entry_point = entry_point;
        self
    }

    pub fn build(&self) -> FragmentState<'fsb>{
        FragmentState{
            color_target_states: self.color_target_states.clone(),
            entry_point: self.entry_point,
            shader: self.shader,
        }
    }

}

///
/// Layout of the VertexState of a Pipeline.
/// It describes the buffer layouts as well as the names used when setting by name in the 
/// RenderPassPipeline process.
///
pub struct VertexState<'vs>{
    pub vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'vs>>,
    /// used to save names and corresponding indices.
    pub vertex_buffer_names: Arc<HashMap<String, usize>>,
    pub entry_point: &'vs str,
    pub vertex_shader: &'vs wgpu::ShaderModule,
}

pub struct VertexStateBuilder<'vsb>{
    vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'vsb>>,
    vertex_buffer_names: HashMap<String, usize>,
    entry_point: &'vsb str,
    //module: &'vsb wgpu::ShaderModule,
    vertex_shader: &'vsb wgpu::ShaderModule,
    index: usize,
}

impl <'vsb> VertexStateBuilder<'vsb>{
    pub fn new(vertex_shader: &'vsb wgpu::ShaderModule) -> Self{
        Self{
            vertex_buffer_layouts: Vec::new(),
            vertex_buffer_names: HashMap::new(),
            entry_point: "vs_main",
            index: 0,
            vertex_shader,
        }
    }

    // TODO: create shader in build.
    /*
    pub fn new_from_src_glsl<'s>(device: &wgpu::Device, src: &'s str) -> Self{
        let vertex_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor{
            label: None,
            source: wgpu::ShaderSource::Glsl{
                shader: Cow::from(src),
                stage: naga::ShaderStage::Vertex,
                defines: naga::FastHashMap::default()
            }
        });

        Self{
            vertex_buffer_layouts: Vec::new(),
            vertex_buffer_names: HashMap::new(),
            entry_point: "vs_main",
            index: 0,
            vertex_shader: &vertex_shader,
        }
    }
    */

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

    pub fn build(&self) -> VertexState<'vsb>{
        VertexState{
            vertex_buffer_names: Arc::new(self.vertex_buffer_names.clone()),
            vertex_buffer_layouts: self.vertex_buffer_layouts.clone(),
            entry_point: self.entry_point,
            vertex_shader: self.vertex_shader,
        }
    }
}


pub trait RenderData{
    fn create_bind_group_layout(device: &wgpu::Device) -> Vec<binding::BindGroupLayoutWithDesc>;
    fn get_bind_groups(&self) -> Vec<&wgpu::BindGroup>;
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

pub struct RenderPassPipeline<'rp, 'rpr>{
    pub render_pass: &'rpr mut RenderPass<'rp>,
    pub pipeline: &'rp RenderPipeline,
}

impl<'rp, 'rpr> RenderPassPipeline<'rp, 'rpr>{
    pub fn set_bind_group(&mut self, name: &str, bind_group: &'rp wgpu::BindGroup, offsets: &'rp [wgpu::DynamicOffset]){
        self.render_pass.render_pass.set_bind_group(
            self.pipeline.bind_group_names[name] as u32, 
            bind_group, offsets
        );
    }

    pub fn set_bind_groups(&mut self, bind_groups: &[&'rp wgpu::BindGroup]){
        for (i, bind_group) in bind_groups.iter().enumerate(){
            self.render_pass.render_pass.set_bind_group(
                i as u32,
                bind_group,
                &[],
            )
        }
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


    pub fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>){
        self.render_pass.render_pass.draw(vertices, instances);
    }

    pub fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>){
        self.render_pass.render_pass.draw_indexed(indices, base_vertex, instances);
    }

    pub fn set_pipeline(&'rpr mut self, pipeline: &'rp RenderPipeline) -> Self{
        self.render_pass.render_pass.set_pipeline(&pipeline.pipeline);
        Self{
            render_pass: self.render_pass,
            pipeline,
        }
    }
}

///
/// A Render Pass with a Pipeline.
///
/// used to reference that Pipeline so one is able to set the bind groups and vertex buffers by
/// name.
///
/// Not quite sure about lifetime inheritance.

pub struct RenderPass<'rp>{
    pub render_pass: wgpu::RenderPass<'rp>,
}

impl<'rp> RenderPass<'rp>{
    pub fn set_pipeline(&mut self, pipeline: &'rp RenderPipeline) -> RenderPassPipeline<'rp, '_>{
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

pub fn shader_load(device: &wgpu::Device, path: &str, stage: naga::ShaderStage, label: Option<&str>) -> Result<wgpu::ShaderModule>{
    let mut f = File::open(path)?;
    let metadata = fs::metadata(path)?;
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer)?;
    let src = str::from_utf8(&buffer)?;


    let extension = Path::new(path).extension().ok_or(anyhow!("No extension"))?;


    let source = match extension.to_str().ok_or(anyhow!("string conversion"))?{
        "glsl" => wgpu::ShaderSource::Glsl{
            shader: Cow::from(src),
            stage,
            defines: naga::FastHashMap::default()
        },
        "wgsl" => wgpu::ShaderSource::Wgsl(Cow::from(src)),
        _ => return Err(anyhow!("Unknown Extension")),
    };

    Ok(device.create_shader_module(&wgpu::ShaderModuleDescriptor{
        label,
        source,
    }))
}

pub fn shader_with_shaderc(device: &wgpu::Device, src: &str, kind: shaderc::ShaderKind, entry_point: &str, label: Option<&str>) -> Result<wgpu::ShaderModule>{

    let mut compiler = shaderc::Compiler::new().ok_or(anyhow!("error creating compiler"))?;
    let mut options = shaderc::CompileOptions::new().ok_or(anyhow!("error creating shaderc options"))?;

    options.set_warnings_as_errors();
    options.set_target_env(shaderc::TargetEnv::Vulkan, 0);
    options.set_optimization_level(shaderc::OptimizationLevel::Performance);
    options.set_generate_debug_info();

    let spirv = match label{
        Some(label) => compiler.compile_into_spirv(src, kind, label, entry_point, None)?,
        _ => compiler.compile_into_spirv(src, kind, "no_label", entry_point, None)?,
    };

    let module = device.create_shader_module(&wgpu::ShaderModuleDescriptor{
        label,
        source: wgpu::ShaderSource::SpirV(Cow::from(spirv.as_binary()))
    });

    Ok(module)
}




pub struct RenderPipelineBuilder<'rpb>{
    //shader: wgpu::ShaderModule,
    vertex_module: Option<&'rpb wgpu::ShaderModule>,
    fragment_module: Option<&'rpb wgpu::ShaderModule>,
}

impl<'rpb> RenderPipelineBuilder<'rpb>{

}

// TODO:
// Counting RenderPass
