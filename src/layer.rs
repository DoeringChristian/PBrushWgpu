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

    translation: cgmath::Vector3<f32>,
    scale: cgmath::Vector3<f32>,
    rotation: cgmath::Vector4<f32>,

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

        let drawable = Box::new(Model::<Vert2>::new(device, &Vert2::QUAD_VERTS, &Vert2::QUAD_IDXS)?);

        let render_pipeline = program::new(
            &device,
            include_str!("shaders/forward_model.wgsl"),
            *format,
            &[&texture.bind_group_layout.layout, &drawable.uniform_buffer.binding_group_layout.layout],
            &[drawable.vert_buffer_layout()]
        )?;

        let translation = cgmath::Vector3::new(0.0, 0.0, 0.0);
        let scale = cgmath::Vector3::new(1.0, 1.0, 1.0);
        let rotation = cgmath::Vector4::new(0.0, 0.0, 1.0, 0.0);

        Ok(Self{
            texture,
            render_pipeline,
            drawable,
            blendop,
            translation,
            scale,
            rotation,
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
            include_str!("shaders/forward_model.wgsl"),
            *format,
            &[&texture.bind_group_layout.layout, &drawable.uniform_buffer.binding_group_layout.layout],
            &[drawable.vert_buffer_layout()]
        )?;

        let translation = cgmath::Vector3::new(0.0, 0.0, 0.0);
        let scale = cgmath::Vector3::new(1.0, 1.0, 1.0);
        let rotation = cgmath::Vector4::new(0.0, 0.0, 1.0, 0.0);

        Ok(Self{
            texture,
            render_pipeline,
            drawable,
            blendop,
            translation,
            scale,
            rotation,
        })
    }

    pub fn draw(&self, encoder: &mut wgpu::CommandEncoder, queue: &wgpu::Queue, dst: &wgpu::TextureView, dst_size: [u32; 2]) -> Result<()>{
        //self.blendop.draw(encoder, dst, &self.texture.bind_group, &itex.bind_group)?;

        let rot = cgmath::Matrix4::from_axis_angle(cgmath::Vector3::new(self.rotation.x, self.rotation.y, self.rotation.z), self.rotation.w.into());
        let scale = cgmath::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);
        let translation = cgmath::Matrix4::from_translation(self.translation);

        let dst_size_f32 = [dst_size[0] as f32, dst_size[1] as f32];
        let proj: [[f32; 4]; 4] = glm::ortho(-dst_size_f32[0]/2.0, dst_size_f32[0]/2.0, dst_size_f32[1]/2.0, -dst_size_f32[1]/2.0, -1.0, 1.0).into();
        
        self.drawable.model_transforms.model = (translation * scale * rot).into();

        let mut render_pass = dst.render_pass_clear(encoder, None)?;

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.texture.bind_group, &[]);

        self.drawable.draw(queue, &mut render_pass);

        Ok(())
    }

    pub fn blendop(&self) -> Arc<BlendOp>{
        self.blendop.clone()
    }
}
