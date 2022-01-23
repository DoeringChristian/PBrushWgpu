use crate::buffer::*;
use wgpu::util::DeviceExt;

/*
pub trait Uniform: bytemuck::Pod 
+ bytemuck::Zeroable
+ Copy + Clone
{

}

impl<U: Uniform> ToBuffer for U{
    fn create_buffer(&self, device: &wgpu::Device, usage: wgpu::BufferUsages) -> anyhow::Result<wgpu::Buffer> {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[*self]),
            usage,
        });

        Ok(buffer)
    }
}

impl<U: Uniform> ToUniformBuffer for U{
    fn create_uniform_buffer(&self, device: &wgpu::Device) -> anyhow::Result<wgpu::Buffer> {
        self.create_buffer(device, wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST)
    }
}
*/
