use crate::pipeline;
use crate::binding;
use crate::binding::GetBindGroupLayout;
use crate::mesh;
use crate::blendop;
use crate::texture;
use anyhow::*;
use std::sync::Arc;

pub struct Surface{
    drawable: Arc<dyn mesh::Drawable>,
    tex_target: texture::Texture,
    tex_src: texture::Texture,

    render_pipeline: pipeline::RenderPipeline,

    blendop: Arc<blendop::BlendOp>,
}

impl Surface{
    pub fn load(device: &wgpu::Device, queue: &wgpu::Queue, format: &wgpu::TextureFormat, blendop: Arc<blendop::BlendOp>, path: &str) -> Result<Self>{
        todo!()
    }
}
