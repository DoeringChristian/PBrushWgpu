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
    texture_layout: wgpu::BindGroupLayout,
    texture_bind_group: wgpu::BindGroup,

    render_pipeline: wgpu::RenderPipeline,

    mesh: Mesh,
}

impl State for WinState{
    fn new(fstate: &mut FrameworkState) -> Self {

        let texture = texture::Texture::new_black(
            (fstate.size.width, fstate.size.height),
            &fstate.device,
            &fstate.queue,
            None,
            fstate.config.format
        ).unwrap();

        let texture_layout = texture::Texture::create_bind_group_layout(&fstate.device, None).unwrap();
        let texture_bind_group = texture.create_bind_group(&fstate.device, &texture_layout, None).unwrap();

        let render_pipeline = program::Program::new(
            &fstate.device, 
            include_str!("shaders/test01.wgsl"), 
            fstate.config.format, 
            &[], 
            &[Vert2::desc()]
        ).unwrap().render_pipeline;

        let mesh = Mesh::new(&fstate.device, &Vert2::QUAD_VERTS, &Vert2::QUAD_IDXS).unwrap();

        Self{
            texture_layout,
            texture_bind_group,
            render_pipeline,
            mesh,
        }
    }
    fn render(&mut self, fstate: &mut FrameworkState, control_flow: &mut ControlFlow) -> Result<(), wgpu::SurfaceError> {
        let output = fstate.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = fstate.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = view.render_pass(&mut encoder, None).unwrap();

            render_pass.set_pipeline(&self.render_pipeline);
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
