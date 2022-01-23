use crate::blendop;
use crate::layer;
use crate::texture;
use anyhow::*;
use std::cell::RefCell;
use std::sync::Arc;

pub struct Canvas {
    layers: Vec<RefCell<layer::Layer>>,
    blendops: Arc<blendop::BlendOpManager>,
    size: [u32; 2],
    tex_tmp0: texture::Texture,
    tex_tmp1: texture::Texture,
    tex_tmp2: texture::Texture,
}

impl Canvas {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        blendops: Arc<blendop::BlendOpManager>,
        size: [u32; 2],
    ) -> Result<Self> {
        let layers: Vec<RefCell<layer::Layer>> = Vec::new();

        let blendops = blendops;

        let tex_tmp0 = texture::Texture::new_black(size, device, queue, None, format)?;
        let tex_tmp1 = texture::Texture::new_black(size, device, queue, None, format)?;
        let tex_tmp2 = texture::Texture::new_black(size, device, queue, None, format)?;

        Ok(Self {
            layers,
            blendops,
            size,
            tex_tmp0,
            tex_tmp1,
            tex_tmp2,
        })
    }

    pub fn push_layer(&mut self, layer: layer::Layer) {
        self.layers.push(RefCell::new(layer));
    }

    pub fn remove_layer(&mut self, index: usize) {
        self.layers.remove(index);
    }

    pub fn draw(&self, encoder: &mut wgpu::CommandEncoder, dst: &wgpu::TextureView) -> Result<()> {
        for (i, layer) in self.layers.iter().enumerate() {
            layer.borrow().draw(encoder, &self.tex_tmp0.view)?;
            let blendop = layer.borrow().blendop();

            if i == self.layers.len() - 1 {
                if i % 2 == 0 {
                    blendop.draw(
                        encoder,
                        dst,
                        &self.tex_tmp0.bind_group,
                        &self.tex_tmp2.bind_group,
                    )?;
                } else {
                    blendop.draw(
                        encoder,
                        dst,
                        &self.tex_tmp0.bind_group,
                        &self.tex_tmp1.bind_group,
                    )?;
                }
            } else {
                if i % 2 == 0 {
                    blendop.draw(
                        encoder,
                        &self.tex_tmp1.view,
                        &self.tex_tmp0.bind_group,
                        &self.tex_tmp2.bind_group,
                    )?;
                } else {
                    blendop.draw(
                        encoder,
                        &self.tex_tmp2.view,
                        &self.tex_tmp0.bind_group,
                        &self.tex_tmp1.bind_group,
                    )?;
                }
            }
        }

        Ok(())
    }
}
