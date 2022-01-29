use device::Device;
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

use std::{sync::Arc, collections::HashMap};

#[macro_use]
extern crate more_asserts;

extern crate nalgebra_glm as glm;
extern crate naga;

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
mod device;

use framework::*;
use binding::*;
use vert::*;
use mesh::*;

struct WinState{
    blendops: Arc<blendop::BlendOpManager>,

    brushops: Arc<brush::BrushOpManager>,

    canvas: canvas::Canvas,

    cursor_prev: [f32; 2],

    devices: HashMap<DeviceId, Device>,
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

        /*
        canvas.push_layer(layer::Layer::load(
                &fstate.device,
                &fstate.queue,
                &fstate.config.format,
                blendops.arc_to("Add").unwrap(),
                "assets/test2.jpg"
        ).unwrap());
        */

        canvas.layers[0].borrow_mut().scale = glm::vec3(300.0, 200.0, 1.0);
        //canvas.layers[1].borrow_mut().scale = glm::vec3(800.0, 800.0, 1.0);

        /*
        canvas.layers[0].borrow_mut().queue_stroke(brush::Stroke::new(
                &fstate.device,
                brushops.arc_to("default").unwrap(),
                brush::StrokeDataUniform{
                    pos0: [0.4, 0.4],
                    pos1: [0.45, 0.45],
                }
        ));
        canvas.layers[0].borrow_mut().queue_stroke(brush::Stroke::new(
                &fstate.device,
                brushops.arc_to("default").unwrap(),
                brush::StrokeDataUniform{
                    pos0: [0.45, 0.45],
                    pos1: [0.5, 0.45],
                }
        ));
        */

        Self{
            blendops,
            brushops,
            canvas,
            cursor_prev: [0.0, 0.0],
            devices: HashMap::new(),
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

    fn cursor_moved(&mut self, fstate: &mut FrameworkState, device_id: &winit::event::DeviceId, position: &winit::dpi::PhysicalPosition<f64>) {
        // have to invert y axis.
        let pos = [position.x as f32 / fstate.size.width as f32, 1.0 - position.y as f32 / fstate.size.height as f32];
        self.canvas.layers[0].borrow_mut().queue_stroke(brush::Stroke::new(
                &fstate.device,
                self.brushops.arc_to("default").unwrap(),
                brush::StrokeDataUniform{
                    pos0: self.cursor_prev,
                    pos1: pos,
                    p0: 1.0,
                    p1: 1.0,
                }
        ));

        self.cursor_prev = pos;
        println!("{:?}, {:?}", device_id, position);
    }

    fn resize(&mut self, fstate: &mut FrameworkState, new_size: winit::dpi::PhysicalSize<u32>){
        self.canvas.resize(&fstate.device, &fstate.queue, [new_size.width, new_size.height]).unwrap();
    }
    fn device_event(&mut self, fstate: &mut FrameworkState, device_id: &winit::event::DeviceId, device_event: &DeviceEvent) {
        let device = self.devices.entry(*device_id).or_insert(Device::default());

        if let DeviceEvent::Motion{axis, value} = device_event{
            if *axis == 0u32{
                device.pos[0] = *value as f32;
            }
            if *axis == 1u32{
                device.pos[1] = *value as f32;
            }
        }
    }
}

fn main() {
    let framework = Framework::<WinState>::new().run();
}
