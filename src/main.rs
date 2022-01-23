use render_target::RenderTarget;
#[allow(unused)]

use winit::{
    event::*,
    event_loop::{
        ControlFlow, 
        EventLoop,
    },
    window::WindowBuilder,
};

use std::sync::Arc;

#[macro_use]
extern crate more_asserts;

mod framework;
mod vert;
mod mesh;
mod texture;
mod render_target;
mod binding;
mod buffer;
mod uniform;
mod program;
mod layer;
mod pipeline;
mod blendop;
mod canvas;

use framework::*;
use binding::*;
use vert::*;
use mesh::*;

struct WinState{
    texture: texture::Texture,
    texture_tmp: texture::Texture,
    layer: layer::Layer,

    render_pipeline: wgpu::RenderPipeline,

    blendops: blendop::BlendOpManager,

    canvas: canvas::Canvas,

    mesh: Mesh<Vert2>,
}

impl State for WinState{
    fn new(fstate: &mut FrameworkState) -> Self {

        let blendops = Arc::new(blendop::BlendOpManager::new(&fstate.device, &fstate.queue, &fstate.config.format).unwrap());

        let canvas = canvas::Canvas::new(&fstate.device, &fstate.queue, fstate.config.format, blendops.clone(), [100, 100]).unwrap();

        let texture = texture::Texture::load_from_path(
            &fstate.device,
            &fstate.queue,
            "assets/test1.jpg",
            Some("texture"),
            fstate.config.format,
        ).unwrap();

        let texture_tmp = texture::Texture::new_black(
            (100, 100),
            &fstate.device,
            &fstate.queue,
            None,
            fstate.config.format,
        ).unwrap();

        let render_pipeline = program::new(
            &fstate.device, 
            include_str!("shaders/forward.wgsl"), 
            fstate.config.format, 
            &[&texture.bind_group_layout.layout], 
            &[Vert2::buffer_layout()]
        ).unwrap();

        let mesh = Mesh::new(&fstate.device, &Vert2::QUAD_VERTS, &Vert2::QUAD_IDXS).unwrap();
        
        let layer = layer::Layer::new(
            &fstate.device,
            &fstate.queue,
            &fstate.config.format,
            (100, 100),
            blendops.arc_to("Add").unwrap()
        ).unwrap();

        Self{
            render_pipeline,
            mesh,
            texture,
            texture_tmp,
            layer,
            blendops,
        }
    }
    fn render(&mut self, fstate: &mut FrameworkState, control_flow: &mut ControlFlow) -> Result<(), wgpu::SurfaceError> {
        let output = fstate.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = fstate.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = self.texture_tmp.view.render_pass_load(&mut encoder, None).unwrap();

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.texture.bind_group, &[]);
            self.mesh.draw(&mut render_pass);
        }

        self.layer.draw(&mut encoder, &view).unwrap();

        /*
        {
            let mut render_pass = view.render_pass(&mut encoder, None).unwrap();

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.texture_tmp.bind_group, &[]);
            self.mesh.draw(&mut render_pass);
        }
        */
        

        fstate.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn input(&mut self, event: &WindowEvent) -> bool{false}

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>){}
}

fn main() {
    let framework = Framework::<WinState>::new().run();
}
