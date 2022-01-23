use crate::vert::*;
use crate::buffer::*;
#[allow(unused)]
use wgpu::util::DeviceExt;
use anyhow::*;
use std::marker::PhantomData;

pub trait Drawable{
    fn draw<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>);
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

    /*
    pub fn draw<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>){
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
    */

    /*
    pub fn desc() -> wgpu::VertexBufferLayout<'static>{
        V::desc()
    }
    */
}

impl<V: Vert> Drawable for Mesh<V>{
    fn draw<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
    fn vert_buffer_layout(&self) -> wgpu::VertexBufferLayout<'static>{
        V::buffer_layout()
    }
}


