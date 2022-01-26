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
mod brush;
mod surface;

use framework::*;
use binding::*;
use vert::*;
use mesh::*;

struct WinState{
    blendops: Arc<blendop::BlendOpManager>,

    brushops: Arc<brush::BrushOpManager>,

    canvas: canvas::Canvas,
}

impl State for WinState{
    fn new(fstate: &mut FrameworkState) -> Self {

        let blendops = Arc::new(blendop::BlendOpManager::new(&fstate.device, &fstate.queue, &fstate.config.format).unwrap());
        let brushops = Arc::new(brush::BrushOpManager::new(&fstate.device, &fstate.queue, fstate.config.format).unwrap());

        let mut canvas = canvas::Canvas::new(&fstate.device, &fstate.queue, fstate.config.format, blendops.clone(), [1000, 1000]).unwrap();

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

        canvas.layers[0].borrow_mut().queue_stroke(brush::Stroke::new(
                &fstate.device,
                brushops.arc_to("default").unwrap(),
                brush::StrokeUniform{
                    pos0: [0.0, 0.0],
                    pos1: [1.0, 1.0],
                }
        ));

        Self{
            blendops,
            brushops,
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

    fn resize(&mut self, fstate: &mut FrameworkState, new_size: winit::dpi::PhysicalSize<u32>){
        self.canvas.resize(&fstate.device, &fstate.queue, [new_size.width, new_size.height]).unwrap();
    }
}

fn main() {
    let framework = Framework::<WinState>::new().run();
}
