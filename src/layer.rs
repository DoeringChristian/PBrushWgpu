use crate::mesh::*;
use crate::program;
use crate::render_target::RenderTarget;
use crate::texture;
use crate::vert::Vert;
use crate::blendop::BlendOp;
use crate::vert::Vert2;
use anyhow::*;
use std::sync::Arc;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct LayerUniform{
    model: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
}

pub struct Layer{
    drawable: Box<dyn Drawable>,
    texture: texture::Texture,
    render_pipeline: wgpu::RenderPipeline,

    blendop: Arc<BlendOp>,
}

impl Layer{

    pub fn load(device: &wgpu::Device, queue: &wgpu::Queue, format: &wgpu::TextureFormat, blendop: Arc<BlendOp>, path: &str) -> Result<Self>{
        let texture = texture::Texture::load_from_path(
            device,
            queue,
            path,
            None,
            *format,
        )?;

        let drawable = Box::new(Mesh::<Vert2>::new(device, &Vert2::QUAD_VERTS, &Vert2::QUAD_IDXS)?);

        let render_pipeline = program::new(
            &device,
            include_str!("shaders/forward.wgsl"),
            *format,
            &[&texture.bind_group_layout.layout],
            &[drawable.vert_buffer_layout()]
        )?;

        Ok(Self{
            texture,
            render_pipeline,
            drawable,
            blendop,
        })
    }

    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: &wgpu::TextureFormat, size: [u32; 2], blendop: Arc<BlendOp>) -> Result<Self>{
        let texture = texture::Texture::new_black(
            size,
            &device,
            &queue,
            None,
            *format,
        )?;

        let drawable = Box::new(Mesh::<Vert2>::new(device, &Vert2::QUAD_VERTS, &Vert2::QUAD_IDXS)?);

        let render_pipeline = program::new(
            &device,
            include_str!("shaders/forward.wgsl"),
            *format,
            &[&texture.bind_group_layout.layout],
            &[drawable.vert_buffer_layout()]
        )?;

        Ok(Self{
            texture,
            render_pipeline,
            drawable,
            blendop,
        })
    }

    pub fn draw(&self, encoder: &mut wgpu::CommandEncoder, dst: &wgpu::TextureView) -> Result<()>{
        //self.blendop.draw(encoder, dst, &self.texture.bind_group, &itex.bind_group)?;

        let mut render_pass = dst.render_pass_clear(encoder, None)?;

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.texture.bind_group, &[]);

        self.drawable.draw(&mut render_pass);

        Ok(())
    }

    pub fn blendop(&self) -> Arc<BlendOp>{
        self.blendop.clone()
    }
}
