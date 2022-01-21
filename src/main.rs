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

mod framework;
mod vert;
mod mesh;
mod texture;
mod render_target;
mod bindable;
mod bufferable;
mod uniform;
mod program;

use framework::*;
use bindable::*;
use vert::*;
use mesh::*;

struct WinState{
    texture: texture::Texture,
    texture_tmp: texture::Texture,

    render_pipeline: wgpu::RenderPipeline,

    mesh: Mesh,
}

impl State for WinState{
    fn new(fstate: &mut FrameworkState) -> Self {

        let texture = texture::Texture::from_bytes(
            &fstate.device,
            &fstate.queue,
            include_bytes!("imgs/tree.png"),
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

        let render_pipeline = program::Program::new(
            &fstate.device, 
            include_str!("shaders/forward.wgsl"), 
            fstate.config.format, 
            &[&texture.bind_group_layout], 
            &[Vert2::desc()]
        ).unwrap().render_pipeline;

        let mesh = Mesh::new(&fstate.device, &Vert2::QUAD_VERTS, &Vert2::QUAD_IDXS).unwrap();

        Self{
            render_pipeline,
            mesh,
            texture,
            texture_tmp,
        }
    }
    fn render(&mut self, fstate: &mut FrameworkState, control_flow: &mut ControlFlow) -> Result<(), wgpu::SurfaceError> {
        let output = fstate.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = fstate.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = self.texture_tmp.view.render_pass(&mut encoder, None).unwrap();

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.texture.bind_group, &[]);
            self.mesh.draw(&mut render_pass);
        }

        {
            let mut render_pass = view.render_pass(&mut encoder, None).unwrap();

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.texture_tmp.bind_group, &[]);
            self.mesh.draw(&mut render_pass);
        }

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
