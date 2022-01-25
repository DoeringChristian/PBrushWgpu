use std::collections::HashMap;
use std::sync::Arc;
use anyhow::*;
use crate::buffer;
use crate::mesh;
use crate::mesh::Drawable;
use crate::vert;
use crate::program;
use crate::texture;
use crate::pipeline;
use crate::render_target::RenderTarget;
use crate::binding::ToBindGroupLayout;


pub struct BrushOp{
    render_pipeline: pipeline::RenderPipeline,
    drawable: Arc<dyn mesh::Drawable>,
}

impl BrushOp{
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, src: &str) -> Result<Self>{
        // TODO: Should use a global mesh.
        let drawable = Arc::new(mesh::Mesh::<vert::Vert2>::new(
                device, &vert::Vert2::QUAD_VERTS, 
                &vert::Vert2::QUAD_IDXS
        )?);

        let texture_bgl = texture::Texture::create_bind_group_layout(device, None);
        let uniform_bgl = buffer::UniformBindGroup::<StrokeUniform>::create_bind_group_layout(device, None);

        let render_pipeline_layout = pipeline::PipelineLayoutBuilder::new()
            .push_named("background", &texture_bgl)
            .push_named("self", &texture_bgl)
            .push_named("stroke", &uniform_bgl)
            .create(device, None);

        let render_pipeline = program::new(
            &device,
            src,
            format,
            &render_pipeline_layout,
            &[drawable.vert_buffer_layout()]
        )?;

        Ok(Self{
            render_pipeline,
            drawable,
        })
    }

    // TODO: change bind_groups to render_pass.
    pub fn draw<'rp>(&'rp self, render_pass: &mut pipeline::RenderPass<'rp>) -> Result<()>{

        // TODO: move out of basic drawing function.
        //let mut render_pass = dst.render_pass_clear(encoder, None)?;

        render_pass.set_pipeline(&self.render_pipeline);

        self.drawable.draw(render_pass);

        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct StrokeUniform{
    pub pos0: [f32; 2],
    pub pos1: [f32; 2],
}

pub struct Stroke{
    brushop: Arc<BrushOp>,
    pub uniform: buffer::UniformBindGroup<StrokeUniform>,
}

impl Stroke{
    pub fn new(device: &wgpu::Device, brushop: Arc<BrushOp>, suniform: StrokeUniform) -> Self{


        let uniform = buffer::UniformBindGroup::new_with_data(device, &suniform);

        Self{
            brushop,
            uniform,
        }
    }

    pub fn draw<'rp>(&'rp self, render_pass: &mut pipeline::RenderPass<'rp>) -> Result<()>{

        render_pass.set_bind_group(2, &self.uniform.binding_group, &[]);
        
        self.brushop.draw(render_pass)?;

        Ok(())
    }
}

pub struct BrushOpManager{
    ops: HashMap<String, Arc<BrushOp>>,
}

impl BrushOpManager{
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Result<Self>{
        let mut ops: HashMap<String, Arc<BrushOp>> = HashMap::new();

        let brushop_default = BrushOp::new(device, format, include_str!("shaders/brush01.wgsl"))?;

        ops.insert("default".to_string(), Arc::new(brushop_default));

        Ok(Self{
            ops,
        })
    }

    pub fn arc_to(&self, key: &str) -> Result<Arc<BrushOp>>{
        Ok(self.ops.get(key).ok_or(anyhow!("No BrushOp found for this name"))?.clone())
    }
}



