#[allow(unused)]
use winit::{
    event::*,
    event_loop::{
        ControlFlow,
        EventLoop,
    },
    window::{
        Window,
        WindowBuilder,
    },
};

pub trait State{
    fn new(fstate: &mut FrameworkState) -> Self;
    fn render(&mut self, fstate: &mut FrameworkState, control_flow: &mut ControlFlow) -> Result<(), wgpu::SurfaceError>{Ok(())}
    fn input(&mut self, event: &WindowEvent) -> bool{false}
    fn cursor_moved(&mut self, fstate: &mut FrameworkState, device_id: &winit::event::DeviceId, position: &winit::dpi::PhysicalPosition<f64>){}
    fn device_event(&mut self, fstate: &mut FrameworkState, device_id: &winit::event::DeviceId, device_event: &DeviceEvent){}
    fn resize(&mut self, fstate: &mut FrameworkState, new_size: winit::dpi::PhysicalSize<u32>){}
}

pub struct FrameworkState{
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Window,
}

impl FrameworkState{
    pub async fn new(window: Window) -> Self{
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe{instance.create_surface(&window)};
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions{
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor{
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ).await.unwrap();
        let config = wgpu::SurfaceConfiguration{
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        Self{
            surface,
            device,
            queue,
            config,
            size,
            window,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>){
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}

pub struct Framework<S: State>{
    fstate: FrameworkState,
    state: S,
    event_loop: EventLoop<()>,
}

impl<S: 'static +  State> Framework<S>{

    pub fn new() -> Self{
        env_logger::init();

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(winit::dpi::LogicalSize::new(1000, 600))
            .build(&event_loop).unwrap();

        let mut fstate = pollster::block_on(FrameworkState::new(window));

        let state = S::new(&mut fstate);

        Self{
            fstate,
            state,
            event_loop,
        }
    }

    pub fn run(mut self){
        self.event_loop.run(move |event, _, control_flow|{
            match event{
                Event::WindowEvent{
                    ref event,
                    window_id,
                } if window_id == self.fstate.window.id() => if !self.state.input(event){
                    match event{
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            self.fstate.resize(*physical_size);
                            self.state.resize(&mut self.fstate, *physical_size);
                        },

                        WindowEvent::ScaleFactorChanged{new_inner_size, ..} => {
                            self.fstate.resize(**new_inner_size);
                            self.state.resize(&mut self.fstate, **new_inner_size);
                        },
                        WindowEvent::CursorMoved{device_id, position, ..} => {
                            self.state.cursor_moved(&mut self.fstate, device_id, position);
                        }
                        _ => {},
                    }
                },

                Event::RedrawRequested(window_id) if window_id == self.fstate.window.id() => {
                    match self.state.render(&mut self.fstate, control_flow){
                        Ok(_) => {}

                        Err(wgpu::SurfaceError::Lost) => self.fstate.resize(self.fstate.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,

                        Err(e) => eprintln!("{:?}", e),
                    }
                },
                Event::DeviceEvent{device_id, event} => {
                    self.state.device_event(&mut self.fstate, &device_id, &event);
                    println!("{:?}, {:?}", device_id, event);
                }

                Event::MainEventsCleared => {
                    self.fstate.window.request_redraw();
                },
                _ => {}
            }
        });
    }
}
