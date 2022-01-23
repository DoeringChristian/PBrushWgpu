use crate::binding;
use crate::vert::*;
use crate::buffer::*;
use bytemuck;
use cgmath::*;
#[allow(unused)]
use wgpu::util::DeviceExt;
use anyhow::*;
use std::marker::PhantomData;

pub trait Drawable{
    fn draw<'rp>(&'rp self, queue: &wgpu::Queue, render_pass: &mut wgpu::RenderPass<'rp>);
    fn vert_buffer_layout(&self) -> wgpu::VertexBufferLayout<'static>;
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
    fn draw<'rp>(&'rp self, queue: &wgpu::Queue, render_pass: &mut wgpu::RenderPass<'rp>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
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
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    _phantom_data: PhantomData<V>,
    pub model_transforms: ModelTransforms,
    pub uniform_buffer: UniformBuffer<ModelTransforms>,
}

impl<V: Vert> Model<V>{
    pub fn new(device: &wgpu::Device, verts: &[V], idxs: &[u32]) -> Result<Self>{
        let vertex_buffer = verts.create_vert_buffer(device)?;

        let index_buffer = idxs.create_idx_buffer(device)?;

        let num_indices = idxs.len() as u32;

        let model = cgmath::Matrix4::identity();
        let view = cgmath::Matrix4::identity();
        let proj = cgmath::Matrix4::identity();
        let model_transforms = ModelTransforms{
            model: model.into(),
            view: view.into(),
            proj: proj.into(),
        };
        let uniform_buffer = UniformBuffer::new_with_data(device, &model_transforms);

        Ok(Self{
            vertex_buffer,
            index_buffer,
            num_indices,
            _phantom_data: PhantomData,
            uniform_buffer,
            model_transforms,
        })
    }
}

impl<V: Vert> Drawable for Model<V>{
    fn draw<'rp>(&'rp self, queue: &wgpu::Queue, render_pass: &mut wgpu::RenderPass<'rp>) {

        self.uniform_buffer.update(queue, &self.model_transforms);

        render_pass.set_bind_group(1, &self.uniform_buffer.binding_group, &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
    fn vert_buffer_layout(&self) -> wgpu::VertexBufferLayout<'static>{
        V::buffer_layout()
    }
}

