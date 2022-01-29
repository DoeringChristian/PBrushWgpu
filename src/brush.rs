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
use crate::layer;
use crate::binding::GetBindGroup;
use crate::render_target::RenderTarget;
use crate::binding::ToBindGroupLayout;


pub struct BrushOp{
    render_pipeline: pipeline::RenderPipeline,
    drawable: Arc<dyn mesh::Drawable>,
}

pub struct BrushBindGroups<'bg>{
    stroke_data: StrokeBindGroups<'bg>,
    stroke: &'bg buffer::UniformBindGroup<StrokeDataUniform>,
    transforms: &'bg buffer::UniformBindGroup<mesh::ModelTransforms>,
}

/// Add an iter for layouts and bindgroups
impl<'bg> BrushBindGroups<'bg>{
    pub fn set_bind_groups(&self, render_pass_pipeline: &mut pipeline::RenderPassPipeline<'bg, '_>){
        render_pass_pipeline.set_bind_group("transforms", self.transforms.get_bind_group(), &[]);
        render_pass_pipeline.set_bind_group("background", self.stroke_data.background,      &[]);
        render_pass_pipeline.set_bind_group("self",       self.stroke_data.tex_self,        &[]);
        render_pass_pipeline.set_bind_group("stroke",     self.stroke.get_bind_group(),     &[]);
    }
}

impl BrushOp{
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, src: &str) -> Result<Self>{
        // TODO: Should use a global mesh.
        let drawable = Arc::new(mesh::Mesh::<vert::Vert2>::new(
                device, &vert::Vert2::QUAD_VERTS, 
                &vert::Vert2::QUAD_IDXS
        )?);

        let texture_bgl = texture::Texture::create_bind_group_layout(device, None);
        let stroke_uniform_bgl = buffer::UniformBindGroup::<StrokeDataUniform>::create_bind_group_layout(device, None);
        let transforms_uniform_bgl = buffer::UniformBindGroup::<mesh::ModelTransforms>::create_bind_group_layout(device, None);

        let render_pipeline_layout = pipeline::PipelineLayoutBuilder::new()
            .push_named("transforms", &transforms_uniform_bgl)
            .push_named("self", &texture_bgl)
            .push_named("stroke", &stroke_uniform_bgl)
            .push_named("background", &texture_bgl)
            .create(device, None);

        let vert_shader = pipeline::shader_with_shaderc(device, include_str!("shaders/vert_brush.glsl"), shaderc::ShaderKind::Vertex, "main", Some("VertexShader"))?;
        let frag_shader = pipeline::shader_with_shaderc(device, include_str!("shaders/frag_brush01.glsl"), shaderc::ShaderKind::Fragment, "main", Some("FragmentShader"))?;

        let vertex_state = pipeline::VertexStateBuilder::new(&vert_shader)
            .push_named("model", drawable.vert_buffer_layout())
            .set_entry_point("main")
            .build();

        let fragment_state = pipeline::FragmentStateBuilder::new(&frag_shader)
            .set_entry_point("main")
            .build();

        let render_pipeline = program::new(
            &device,
            format,
            &render_pipeline_layout,
            &vertex_state,
            &fragment_state,
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

impl<'pd> mesh::DataDrawable<'pd, BrushBindGroups<'pd>> for BrushOp{
    fn draw_data(&'pd self, render_pass: &'_ mut pipeline::RenderPass<'pd>, data: BrushBindGroups<'pd>){
        let mut render_pass_pipeline = render_pass.set_pipeline(&self.render_pipeline);

        data.set_bind_groups(&mut render_pass_pipeline);
        /*
        render_pass_pipeline.set_bind_group("transforms", data.transforms.get_bind_group(), &[]);
        render_pass_pipeline.set_bind_group("background", data.stroke_data.background, &[]);
        render_pass_pipeline.set_bind_group("self", data.stroke_data.tex_self, &[]);
        render_pass_pipeline.set_bind_group("stroke", data.stroke.get_bind_group(), &[]);
        */
        
        self.drawable.draw(&mut render_pass_pipeline);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct StrokeDataUniform{
    pub pos0: [f32; 2],
    pub pos1: [f32; 2],
    pub p0: f32,
    pub p1: f32,
}

pub struct StrokeBindGroups<'bg>{
    pub background: &'bg wgpu::BindGroup,
    pub tex_self: &'bg wgpu::BindGroup,
}

pub struct Stroke{
    brushop: Arc<BrushOp>,
    pub data_uniform: buffer::UniformBindGroup<StrokeDataUniform>,
    pub transforms_uniform: buffer::UniformBindGroup<mesh::ModelTransforms>,
}

impl Stroke{
    pub fn new(device: &wgpu::Device, brushop: Arc<BrushOp>, suniform: StrokeDataUniform) -> Self{
        let data_uniform = buffer::UniformBindGroup::new_with_data(device, &suniform);
        let transforms_uniform = buffer::UniformBindGroup::new(device);

        Self{
            brushop,
            data_uniform,
            transforms_uniform,
        }
    }

    pub fn update_transforms(&mut self, queue: &wgpu::Queue, transforms_uniform: &mesh::ModelTransforms){
        self.transforms_uniform.update(queue, transforms_uniform);
    }

}

impl<'pd> mesh::DataDrawable<'pd, StrokeBindGroups<'pd>> for Stroke{
    fn draw_data(&'pd self, render_pass: &'_ mut pipeline::RenderPass<'pd>, data: StrokeBindGroups<'pd>) {
        self.brushop.draw_data(render_pass, BrushBindGroups{
            stroke_data: data,
            stroke: &self.data_uniform,
            transforms: &self.transforms_uniform,
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



