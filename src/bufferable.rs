#[allow(unused)]
use anyhow::*;
use wgpu::util::DeviceExt;

pub trait Bufferable{
    fn create_buffer(&self, device: &wgpu::Device, usage: wgpu::BufferUsages) -> Result<wgpu::Buffer>;
}

pub trait VertBufferable: Bufferable{
    fn create_vert_buffer(&self, device: &wgpu::Device) -> Result<wgpu::Buffer>;
}

pub trait IdxBufferable: Bufferable{
    fn create_idx_buffer(&self, device: &wgpu::Device) -> Result<wgpu::Buffer>;
}

pub trait UniformBufferable: Bufferable{
    fn create_uniform_buffer(&self, device: &wgpu::Device) -> Result<wgpu::Buffer>;
}



impl Bufferable for &[u32]{
    fn create_buffer(&self, device: &wgpu::Device, usage: wgpu::BufferUsages) -> Result<wgpu::Buffer> {
        Ok(device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: None,
            contents: bytemuck::cast_slice(*self),
            usage,
        }))
    }
}

impl IdxBufferable for &[u32]{
    fn create_idx_buffer(&self, device: &wgpu::Device) -> Result<wgpu::Buffer> {
        self.create_buffer(device, wgpu::BufferUsages::INDEX)
    }
}



