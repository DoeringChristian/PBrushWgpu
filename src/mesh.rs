use crate::binding;
use crate::pipeline;
use crate::vert::*;
use crate::buffer::*;
use crate::binding::GetBindGroup;
use bytemuck;
use cgmath::*;
#[allow(unused)]
use wgpu::util::DeviceExt;
use anyhow::*;
use std::marker::PhantomData;

///
/// Drawables can be drawn using a RenderPassPipeline.
///
/// The Function vert_buffer_layout can be used to extract the vertex buffer layout when creating a
/// RenderPipeline.
///
pub trait Drawable{
    fn draw<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>);
    fn vert_buffer_layout(&self) -> wgpu::VertexBufferLayout<'static>;
}

pub trait UpdatedDrawable<D>: Drawable{
    fn update(&mut self, queue: &wgpu::Queue, data: &D);
    fn update_draw<'rp>(&'rp mut self, queue: &wgpu::Queue, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>, data: &D){
        self.update(queue, data);
        self.draw(render_pass)
    }
}

///
/// A DataDrawable is Anything that can be drawn without knowing the pipeline to use own pipeline.
///
/// It has to get data from somewhere for example textures to draw to.
///
pub trait DataDrawable<'pd, D>{
    fn draw_data(&'pd self, render_pass: &'_ mut pipeline::RenderPass<'pd>, data: D);
}

pub struct Mesh<V: Vert>{
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    _phantom_data: PhantomData<V>,
}

impl<V: Vert> Mesh<V>{
    pub fn new(device: &wgpu::Device, verts: &[V], idxs: &[u32]) -> Result<Self>{

        let vertex_buffer = verts.create_vert_buffer(device)?;

        let index_buffer = idxs.create_idx_buffer(device)?;

        let num_indices = idxs.len() as u32;

        Ok(Self{
            vertex_buffer,
            index_buffer,
            num_indices,
            _phantom_data: PhantomData,
        })
    }
}

impl<V: Vert> Drawable for Mesh<V>{
    fn draw<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>) {

        render_pass.set_vertex_buffer("model", self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
    fn vert_buffer_layout(&self) -> wgpu::VertexBufferLayout<'static>{
        V::buffer_layout()
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelTransforms{
    pub model: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
}

pub struct Model<V: Vert>{
    mesh: Mesh<V>,
    uniform_buffer: UniformBindGroup<ModelTransforms>,
}

impl<V: Vert> Model<V>{
    pub fn new(device: &wgpu::Device, verts: &[V], idxs: &[u32]) -> Result<Self>{

        let mesh = Mesh::<V>::new(device, verts, idxs)?;

        let model = glm::Mat4::identity();
        let view = glm::Mat4::identity();
        let proj = glm::Mat4::identity();
        let model_transforms = ModelTransforms{
            model: model.into(),
            view: view.into(),
            proj: proj.into()
        };
        let uniform_buffer = UniformBindGroup::new_with_data(device, &model_transforms);

        Ok(Self{
            mesh,
            uniform_buffer,
        })
    }
}

impl<V: Vert> Drawable for Model<V>{
    fn draw<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>) {
        render_pass.set_bind_group("transforms", &self.uniform_buffer.get_bind_group(), &[]);

        self.mesh.draw(render_pass);
    }

    fn vert_buffer_layout(&self) -> wgpu::VertexBufferLayout<'static> {
        V::buffer_layout()
    }
}

impl<V: Vert> UpdatedDrawable<ModelTransforms> for Model<V>{
    fn update(&mut self, queue: &wgpu::Queue, data: &ModelTransforms) {
        self.uniform_buffer.update(queue, &data);
    }
}


