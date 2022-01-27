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
use crate::binding::GetBindGroup;
use crate::render_target::RenderTarget;
use crate::binding::ToBindGroupLayout;


pub struct BrushOp{
    render_pipeline: pipeline::RenderPipeline,
    drawable: Arc<dyn mesh::Drawable>,
}

pub struct BrushOpData<'bg>{
    stroke_data: StrokeData<'bg>,
    stroke: &'bg buffer::UniformBindGroup<StrokeUniform>,
}

/// Add an iter for layouts and bindgroups
impl<'bg> BrushOpData<'bg>{
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

        let vertex_state_layout = pipeline::VertexStateLayoutBuilder::new()
            .push_named("model", drawable.vert_buffer_layout())
            .build();

        let render_pipeline = program::new(
            &device,
            src,
            format,
            &render_pipeline_layout,
            &vertex_state_layout,
        )?;

        Ok(Self{
            render_pipeline,
            drawable,
        })
    }

    // TODO: change bind_groups to render_pass.
    pub fn draw<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>) -> Result<()>{
        self.drawable.draw(render_pass);

        Ok(())
    }

    pub fn get_pipeline(&self) -> &pipeline::RenderPipeline{
        &self.render_pipeline
    }
}

impl<'pd> mesh::DataDrawable<'pd, BrushOpData<'pd>> for BrushOp{
    fn draw_data(&'pd self, render_pass: &'_ mut pipeline::RenderPass<'pd>, data: BrushOpData<'pd>){
        let mut render_pass_pipeline = render_pass.set_pipeline(&self.render_pipeline);

        render_pass_pipeline.set_bind_group("background", data.stroke_data.background, &[]);
        render_pass_pipeline.set_bind_group("self", data.stroke_data.tex_self, &[]);
        render_pass_pipeline.set_bind_group("stroke", data.stroke.get_bind_group(), &[]);
        
        self.drawable.draw(&mut render_pass_pipeline);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct StrokeUniform{
    pub pos0: [f32; 2],
    pub pos1: [f32; 2],
}

pub struct StrokeData<'bg>{
    pub background: &'bg wgpu::BindGroup,
    pub tex_self: &'bg wgpu::BindGroup,
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

    pub fn draw<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>) -> Result<()>{

        render_pass.set_bind_group("stroke", &self.uniform.binding_group, &[]);
        
        self.brushop.draw(render_pass)?;

        Ok(())
    }

    pub fn get_pipeline(&self) -> &pipeline::RenderPipeline{
        self.brushop.get_pipeline()
    }
}

impl<'pd> mesh::DataDrawable<'pd, StrokeData<'pd>> for Stroke{
    fn draw_data(&'pd self, render_pass: &'_ mut pipeline::RenderPass<'pd>, data: StrokeData<'pd>) {
        self.brushop.draw_data(render_pass, BrushOpData{
            stroke_data: data,
            stroke: &self.uniform,
        });
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



