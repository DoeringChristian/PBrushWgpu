use crate::brush::StrokeBindGroups;
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
use std::borrow::Cow;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformsUniform{
    pub model: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
}

pub struct Layer{
    drawable: Box<dyn UpdatedDrawable<ModelTransforms>>,
    tex_src: texture::Texture,
    // a temporary texture for painting to and from the layer.
    tex_target: texture::Texture,
    render_pipeline: pipeline::RenderPipeline,

    pub translation: glm::Vec3,
    pub scale: glm::Vec3,
    pub rotation: glm::Vec4,
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
            .push_named("transforms", &uniform_buffer.get_bind_group_layout())
            .push_named("src", &texture.bind_group_layout)
            .create(device, None);

        let vertex_shader = pipeline::shader_with_shaderc(device, include_str!("shaders/vert_model.glsl"), shaderc::ShaderKind::Vertex, "main", Some("VertexShader"))?;

        let vertex_state = pipeline::VertexStateBuilder::new(&vertex_shader)
            .push_named("model", drawable.vert_buffer_layout())
            .set_entry_point("main")
            .build();

        let fragment_shader = pipeline::shader_with_shaderc(device, include_str!("shaders/frag_forward.glsl"), shaderc::ShaderKind::Fragment, "main", Some("FragmentShader"))?;

        let fragment_state = pipeline::FragmentStateBuilder::new(&fragment_shader)
            .set_entry_point("main")
            .build();

        let render_pipeline = program::new(
            &device,
            *format,
            &render_pipeline_layout,
            &vertex_state,
            &fragment_state,
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
            .push_named("transforms", &uniform_buffer.get_bind_group_layout())
            .push_named("src", &texture.bind_group_layout)
            .create(device, None);

        let vertex_shader = pipeline::shader_with_shaderc(device, include_str!("shaders/vert_model.glsl"), shaderc::ShaderKind::Vertex, "main", Some("VertexShader"))?;

        let vertex_state = pipeline::VertexStateBuilder::new(&vertex_shader)
            .push_named("model", drawable.vert_buffer_layout())
            .set_entry_point("main")
            .build();

        let fragment_shader = pipeline::shader_with_shaderc(device, include_str!("shaders/frag_forward.glsl"), shaderc::ShaderKind::Fragment, "main", Some("FragmentShader"))?;

        let fragment_state = pipeline::FragmentStateBuilder::new(&fragment_shader)
            .set_entry_point("main")
            .build();

        let render_pipeline = program::new(
            &device,
            *format,
            &render_pipeline_layout,
            &vertex_state,
            &fragment_state,
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
        let size_vec_norm = size_vec;
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
            .begin(encoder, None);
        let mut render_pass_pipeline = render_pass.set_pipeline(&self.render_pipeline);

        render_pass_pipeline.set_bind_group("src", &self.tex_src.bind_group, &[]);

        self.drawable.update(queue, &model_transforms);

        self.drawable.draw(&mut render_pass_pipeline);

        Ok(())
    }

    pub fn blendop(&self) -> Arc<BlendOp>{
        self.blendop.clone()
    }

    pub fn queue_stroke(&mut self, stroke: brush::Stroke){
        self.strokes.push_back(stroke);
    }

    pub fn apply_strokes(&mut self, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, prev: &wgpu::BindGroup, view: [u32; 2]) -> Result<()>{

        let axisv = glm::vec3(self.rotation.x, self.rotation.y, self.rotation.z);
        let axis: nalgebra::Unit<glm::Vec3> = nalgebra::Unit::new_normalize(axisv);
        let rot = glm::Mat4::from_axis_angle(&axis, self.rotation[3]);
        let scale = glm::Mat4::new_nonuniform_scaling(&self.scale);
        let translation = glm::Mat4::new_translation(&self.translation);

        let size_vec = glm::vec2(view[0] as f32, view[1] as f32);
        let size_vec_norm = size_vec;
        let proj_mat4 = glm::ortho(-size_vec_norm[0]/2.0, size_vec_norm[0]/2.0, size_vec_norm[1]/2.0, -size_vec_norm[1]/2.0, -1.0, 1.0);
        let proj: [[f32; 4]; 4] = proj_mat4.into();
        let view: [[f32; 4]; 4] = glm::Mat4::identity().into();

        let model_mat4 = (translation * scale) * rot;
        let model = model_mat4.into();
        let model_transforms = ModelTransforms{
            model,
            view,
            proj,
        };


        for stroke in &mut self.strokes{
            {
                let mut render_pass = pipeline::RenderPassBuilder::new()
                    .push_color_attachment(self.tex_target.view.color_attachment_clear())
                    .begin(encoder, None);

                stroke.update_transforms(queue, &model_transforms);

                stroke.draw_bind_groups(&mut render_pass, StrokeBindGroups{
                    background: prev,
                    tex_self: &self.tex_src.bind_group,
                });
            }

            self.tex_target.copy_all_to(&mut self.tex_src, encoder);
        }

        self.strokes.clear();

        Ok(())
    }
}
