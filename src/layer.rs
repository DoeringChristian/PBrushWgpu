use crate::mesh;
use crate::mesh::*;
use crate::program;
use crate::render_target::ColorAttachment;
use crate::render_target::RenderTarget;
use crate::texture;
use crate::vert::Vert;
use crate::buffer;
use crate::blendop::BlendOp;
use crate::vert::Vert2;
use crate::pipeline;
use crate::binding::GetBindGroupLayout;
use crate::brush;
use anyhow::*;
use std::collections::VecDeque;
use std::sync::Arc;
use crate::binding;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct LayerUniform{
    model: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
}

pub struct Layer{
    drawable: Box<dyn UpdatedDrawable<ModelTransforms>>,
    tex_src: texture::Texture,
    // a temporary texture for painting to and from the layer.
    tex_target: texture::Texture,
    render_pipeline: pipeline::RenderPipeline,

    translation: glm::Vec3,
    scale: glm::Vec3,
    rotation: glm::Vec4,
    uniform_buffer: buffer::UniformBindGroup<ModelTransforms>,

    blendop: Arc<BlendOp>,

    strokes: VecDeque<brush::Stroke>,
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

        let tex_tmp = texture::Texture::new_black(
            texture.size,
            device,
            queue,
            None,
            *format
        )?;

        let drawable = Box::new(Model::<Vert2>::new(device, &Vert2::QUAD_VERTS, &Vert2::QUAD_IDXS)?);

        let model = glm::Mat4::identity();
        let view = glm::Mat4::identity();
        let proj = glm::Mat4::identity();
        let model_transforms = ModelTransforms{
            model: model.into(),
            view: view.into(),
            proj: proj.into()
        };
        let uniform_buffer = buffer::UniformBindGroup::new_with_data(device, &model_transforms);

        let render_pipeline_layout = pipeline::PipelineLayoutBuilder::new()
            .push_named("src", &texture.bind_group_layout)
            .push_named("transforms", &uniform_buffer.get_bind_group_layout())
            .create(device, None);

        let vertex_state = pipeline::VertexStateLayoutBuilder::new()
            .push_named("model", drawable.vert_buffer_layout())
            .build();

        let render_pipeline = program::new(
            &device,
            include_str!("shaders/forward_model.wgsl"),
            *format,
            &render_pipeline_layout,
            &vertex_state,
        )?;

        let translation = glm::vec3(0.0, 0.0, 0.0);
        let scale = glm::vec3(1.0, 1.0, 1.0);
        let rotation = glm::vec4(0.0, 0.0, 1.0, 0.0);

        let strokes: VecDeque<brush::Stroke> = VecDeque::new();

        Ok(Self{
            tex_src: texture,
            render_pipeline,
            drawable,
            uniform_buffer,
            blendop,
            translation,
            scale,
            rotation,
            tex_target: tex_tmp,
            strokes,
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
        let tex_tmp = texture::Texture::new_black(
            texture.size,
            device,
            queue,
            None,
            *format
        )?;

        let drawable = Box::new(Model::<Vert2>::new(device, &Vert2::QUAD_VERTS, &Vert2::QUAD_IDXS)?);

        let model = glm::Mat4::identity();
        let view = glm::Mat4::identity();
        let proj = glm::Mat4::identity();
        let model_transforms = ModelTransforms{
            model: model.into(),
            view: view.into(),
            proj: proj.into()
        };
        let uniform_buffer = buffer::UniformBindGroup::new_with_data(device, &model_transforms);

        let render_pipeline_layout = pipeline::PipelineLayoutBuilder::new()
            .push_named("src", &texture.bind_group_layout)
            .push_named("transforms", &uniform_buffer.get_bind_group_layout())
            .create(device, None);

        let vertex_state = pipeline::VertexStateLayoutBuilder::new()
            .push_named("model", drawable.vert_buffer_layout())
            .build();

        let render_pipeline = program::new(
            &device,
            include_str!("shaders/forward_model.wgsl"),
            *format,
            &render_pipeline_layout,
            &vertex_state,
        )?;

        let translation = glm::vec3(0.0, 0.0, 0.0);
        let scale = glm::vec3(1000.0, 1000.0, 1000.0);
        let rotation = glm::vec4(0.0, 0.0, 1.0, 0.0);

        let strokes: VecDeque<brush::Stroke> = VecDeque::new();

        Ok(Self{
            tex_src: texture,
            render_pipeline,
            drawable,
            uniform_buffer,
            blendop,
            translation,
            scale,
            rotation,
            tex_target: tex_tmp,
            strokes,
        })
    }

    pub fn draw(&mut self, encoder: &mut wgpu::CommandEncoder, queue: &wgpu::Queue, dst: &wgpu::TextureView, dst_size: [u32; 2]) -> Result<()>{
        //self.blendop.draw(encoder, dst, &self.texture.bind_group, &itex.bind_group)?;


        let axisv = glm::vec3(self.rotation.x, self.rotation.y, self.rotation.z);
        let axis: nalgebra::Unit<glm::Vec3> = nalgebra::Unit::new_normalize(axisv);
        let rot = glm::Mat4::from_axis_angle(&axis, self.rotation[3]);
        let scale = glm::Mat4::new_nonuniform_scaling(&self.scale);
        let translation = glm::Mat4::new_translation(&self.translation);

        let size_vec = glm::vec2(dst_size[0] as f32, dst_size[1] as f32);
        let size_vec_norm = size_vec.normalize();
        let proj: [[f32; 4]; 4] = glm::ortho(-size_vec_norm[0]/2.0, size_vec_norm[0]/2.0, size_vec_norm[1]/2.0, -size_vec_norm[1]/2.0, -1.0, 1.0).into();
        let view: [[f32; 4]; 4] = glm::Mat4::identity().into();

        let model = ((translation * scale) * rot).into();
        let model_transforms = ModelTransforms{
            model,
            view,
            proj,
        };

        self.uniform_buffer.update(queue, &model_transforms);

        let mut render_pass = pipeline::RenderPassBuilder::new()
            .push_color_attachment(dst.color_attachment_clear())
            .begin(encoder, None)
            .set_pipeline(&self.render_pipeline);

        render_pass.set_bind_group("src", &self.tex_src.bind_group, &[]);
        self.drawable.update(queue, &model_transforms);

        self.drawable.draw(&mut render_pass);

        Ok(())
    }

    pub fn blendop(&self) -> Arc<BlendOp>{
        self.blendop.clone()
    }

    pub fn queue_stroke(&mut self, stroke: brush::Stroke){
        self.strokes.push_back(stroke);
    }

    pub fn apply_strokes(&mut self, encoder: &mut wgpu::CommandEncoder, prev: &wgpu::BindGroup) -> Result<()>{
        for stroke in &self.strokes{
            {
                let mut render_pass = pipeline::RenderPassBuilder::new()
                    .push_color_attachment(self.tex_target.view.color_attachment_clear())
                    .begin(encoder, None)
                    .set_pipeline(stroke.get_pipeline());

                render_pass.set_bind_group("background", prev, &[]);
                render_pass.set_bind_group("self", &self.tex_src.bind_group, &[]);

                stroke.draw(&mut render_pass)?;

            }

            {
                encoder.copy_texture_to_texture(
                    wgpu::ImageCopyTexture{
                        texture: &self.tex_target.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::ImageCopyTexture{
                        texture: &self.tex_src.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::Extent3d{
                        width: self.tex_src.size[0],
                        height: self.tex_src.size[1],
                        depth_or_array_layers: 1,
                    }
                );
            }
        }

        self.strokes.clear();

        Ok(())
    }
}
