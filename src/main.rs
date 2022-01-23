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

extern crate nalgebra_glm as glm;

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
mod algebra;

use framework::*;
use binding::*;
use vert::*;
use mesh::*;

struct WinState{
    blendops: Arc<blendop::BlendOpManager>,

    canvas: canvas::Canvas,
}

impl State for WinState{
    fn new(fstate: &mut FrameworkState) -> Self {

        let blendops = Arc::new(blendop::BlendOpManager::new(&fstate.device, &fstate.queue, &fstate.config.format).unwrap());

        let mut canvas = canvas::Canvas::new(&fstate.device, &fstate.queue, fstate.config.format, blendops.clone(), [100, 100]).unwrap();

        canvas.push_layer(layer::Layer::load(
                &fstate.device,
                &fstate.queue,
                &fstate.config.format,
                blendops.arc_to("Add").unwrap(),
                "assets/test1.jpg"
        ).unwrap());

        canvas.push_layer(layer::Layer::load(
                &fstate.device,
                &fstate.queue,
                &fstate.config.format,
                blendops.arc_to("Add").unwrap(),
                "assets/test2.jpg"
        ).unwrap());

        Self{
            blendops,
            canvas,
        }
    }

    fn render(&mut self, fstate: &mut FrameworkState, control_flow: &mut ControlFlow) -> Result<(), wgpu::SurfaceError> {
        let output = fstate.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = fstate.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{
            label: Some("Render Encoder"),
        });

        self.canvas.draw(&mut encoder, &fstate.queue, &view, [fstate.size.width, fstate.size.height]).unwrap();

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
