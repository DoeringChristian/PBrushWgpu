use anyhow::*;
use crate::texture;

///
/// RenderTarget should be implemented for anything that can be rendered to.
///
///
///

pub trait RenderTarget{
    fn render_pass<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder, label: Option<&'a str>) -> Result<wgpu::RenderPass<'a>>;
}

impl RenderTarget for wgpu::TextureView{
    fn render_pass<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder, label: Option<&'a str>) -> Result<wgpu::RenderPass<'a>> {
        Ok(encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
            label,
            color_attachments: &[
                wgpu::RenderPassColorAttachment{
                    view: self,
                    resolve_target: None,
                    ops: wgpu::Operations{
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }
            ],
            depth_stencil_attachment: None
        }))
    }
}

impl RenderTarget for [&wgpu::TextureView]{
    fn render_pass<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder, label: Option<&'a str>) -> Result<wgpu::RenderPass<'a>> {
        let mut color_attachments: Vec<wgpu::RenderPassColorAttachment> = Vec::with_capacity(self.len());

        for view in self{
            color_attachments.push(
                wgpu::RenderPassColorAttachment{
                    view: *view,
                    resolve_target: None,
                    ops: wgpu::Operations{
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }
            );
        }

        Ok(encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
            label,
            color_attachments: &&color_attachments,
            depth_stencil_attachment: None,
        }))
    }
}

