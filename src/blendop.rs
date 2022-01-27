use anyhow::*;
use crate::pipeline;
use crate::render_target::ColorAttachment;
use crate::texture;
use crate::mesh;
use crate::mesh::Drawable;
use crate::vert;
use crate::program;
use crate::render_target::RenderTarget;
use crate::binding::ToBindGroupLayout;
use std::collections::HashMap;
use std::sync::Arc;
use std::borrow::Cow;

///
/// A blend op is used to blend two images together.
/// mesh is spanning the whole screen.
/// Eventually move Mesh to some manager because all BlendOps could use the same.
///
pub struct BlendOp{
    drawable: Box<dyn mesh::Drawable>,
    render_pipeline: pipeline::RenderPipeline,
}

impl BlendOp{
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: &wgpu::TextureFormat, src: &str) -> Result<Self>{
        let drawable = Box::new(mesh::Mesh::<vert::Vert2>::new(device, &vert::Vert2::QUAD_VERTS, &vert::Vert2::QUAD_IDXS)?);

        let bind_group_layout = texture::Texture::create_bind_group_layout(device, None);

        let render_pipeline_layout = pipeline::PipelineLayoutBuilder::new()
            .push_named("src", &bind_group_layout)
            .push_named("dst", &bind_group_layout)
            .create(device, None);

        let vert_shader = pipeline::shader_with_shaderc(device, include_str!("shaders/vert_screen.glsl"), shaderc::ShaderKind::Vertex, "main", Some("VertexShader"))?;
        let frag_shader = pipeline::shader_with_shaderc(device, include_str!("shaders/frag_add.glsl"), shaderc::ShaderKind::Fragment, "main", Some("FragmentShader"))?;

        let vertex_state_layout = pipeline::VertexStateBuilder::new(&vert_shader)
            .push_named("model", drawable.vert_buffer_layout())
            .set_entry_point("main")
            .build();

        let fragment_state = pipeline::FragmentStateBuilder::new(&frag_shader)
            .set_entry_point("main")
            .build();

        let render_pipeline = program::new(
            &device,
            *format,
            &render_pipeline_layout,
            &vertex_state_layout,
            &fragment_state,
        )?;

        Ok(Self{
            drawable,
            render_pipeline,
        })
    }

    pub fn draw(&self, encoder: &mut wgpu::CommandEncoder, queue: &wgpu::Queue, dst: &wgpu::TextureView, src0: &wgpu::BindGroup, src1: &wgpu::BindGroup) -> Result<()>{
        {
            let mut render_pass = pipeline::RenderPassBuilder::new()
                .push_color_attachment(dst.color_attachment_clear())
                .begin(encoder, None);
            let mut render_pass_pipeline = render_pass.set_pipeline(&self.render_pipeline);

            render_pass_pipeline.set_bind_group("src", src0, &[]);
            render_pass_pipeline.set_bind_group("dst", src1, &[]);

            self.drawable.draw(&mut render_pass_pipeline);
        }

        Ok(())
    }
}

impl<'pd> mesh::DataDrawable<'pd, (&'pd wgpu::BindGroup, &'pd wgpu::BindGroup)> for BlendOp{
    fn draw_bind_groups(&'pd self, render_pass: &'_ mut pipeline::RenderPass<'pd>, data: (&'pd wgpu::BindGroup, &'pd wgpu::BindGroup)) {
        let mut render_pass_pipeline = render_pass.set_pipeline(&self.render_pipeline);

        render_pass_pipeline.set_bind_group("src", data.0, &[]);
        render_pass_pipeline.set_bind_group("dst", data.1, &[]);

    }
}


pub struct BlendOpManager{
    ops: HashMap<String, Arc<BlendOp>>,
}

impl BlendOpManager{
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: &wgpu::TextureFormat) -> Result<Self>{
        let mut ops: HashMap<String, Arc<BlendOp>> = HashMap::new();

        let blendop_add = BlendOp::new(device, queue, format, include_str!("shaders/add.wgsl"))?;
        ops.insert("Add".to_string(), Arc::new(blendop_add));

        Ok(Self{
            ops,
        })
    }

    pub fn arc_to(&self, key: &str) -> Result<Arc<BlendOp>>{
        Ok(self.ops.get(key).ok_or(anyhow!("No BlendOp found for this name"))?.clone())
    }
}

