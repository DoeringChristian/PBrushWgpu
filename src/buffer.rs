use anyhow::*;
use wgpu::util::DeviceExt;
use std::marker::PhantomData;

pub trait ToBuffer{
    fn create_buffer(&self, device: &wgpu::Device, usage: wgpu::BufferUsages) -> Result<wgpu::Buffer>;
}

pub trait ToVertBuffer: ToBuffer{
    fn create_vert_buffer(&self, device: &wgpu::Device) -> Result<wgpu::Buffer>;
}

pub trait ToIdxBuffer: ToBuffer{
    fn create_idx_buffer(&self, device: &wgpu::Device) -> Result<wgpu::Buffer>;
}

pub trait ToUniformBuffer{
    fn uniform_label() -> &'static str;
    fn create_uniform_buffer(&self, device: &wgpu::Device) -> Result<wgpu::Buffer>;
    fn update_uniform_buffer(&self, queue: &wgpu::Queue, dst: &mut wgpu::Buffer);
}


impl<T: bytemuck::Pod> ToUniformBuffer for T{
    fn uniform_label() -> &'static str{
        let type_name = std::any::type_name::<Self>();
        let pos = type_name.rfind(':').unwrap();
        &type_name[(pos + 1)..]
    }
    fn create_uniform_buffer(&self, device: &wgpu::Device) -> Result<wgpu::Buffer> {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor{
            label: Some(&format!("UniformBuffer: {}", Self::uniform_label())),
            size: std::mem::size_of::<Self>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: true,
        });

        let mapped_memory = buffer.slice(..);
        mapped_memory.get_mapped_range_mut().clone_from_slice(bytemuck::bytes_of(self));

        buffer.unmap();

        Ok(buffer)
    }

    fn update_uniform_buffer(&self, queue: &wgpu::Queue, dst: &mut wgpu::Buffer) {
        queue.write_buffer(&dst, 0, bytemuck::bytes_of(self));
    }
}



impl ToBuffer for &[u32]{
    fn create_buffer(&self, device: &wgpu::Device, usage: wgpu::BufferUsages) -> Result<wgpu::Buffer> {
        Ok(device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: None,
            contents: bytemuck::cast_slice(*self),
            usage,
        }))
    }
}

impl ToIdxBuffer for &[u32]{
    fn create_idx_buffer(&self, device: &wgpu::Device) -> Result<wgpu::Buffer> {
        self.create_buffer(device, wgpu::BufferUsages::INDEX)
    }
}


/// not sure what idiom to use.
pub struct UniformBuffer<C>{
    buffer: wgpu::Buffer,
    content_type: PhantomData<C>,

    content: Vec<u8>,
}

impl<C: bytemuck::Pod> UniformBuffer<C>{
    fn name() -> &'static str{
        let type_name = std::any::type_name::<C>();
        let pos = type_name.rfind(':').unwrap();
        &type_name[(pos + 1)..]
    }
    
    pub fn new(device: &wgpu::Device) -> Self{
        let buffer = device.create_buffer(&wgpu::BufferDescriptor{
            label: Some(&format!("UniformBuffer: {}", Self::name())),
            size: std::mem::size_of::<C>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        UniformBuffer{
            buffer,
            content_type: PhantomData,
            content: Vec::new(),
        }
    }

    pub fn new_with_data(device: &wgpu::Device, src: &C) -> Self{
        let buffer = device.create_buffer(&wgpu::BufferDescriptor{
            label: Some(&format!("UniformBuffer: {}", Self::name())),
            size: std::mem::size_of::<C>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: true,
        });

        let mapped_memory = buffer.slice(..);
        mapped_memory.get_mapped_range_mut().clone_from_slice(bytemuck::bytes_of(src));

        buffer.unmap();

        Self{
            buffer,
            content_type: PhantomData,
            content: bytemuck::bytes_of(src).to_vec(),
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, src: &C){
        let new_content = bytemuck::bytes_of(src);
        if self.content == new_content{
            return;
        }

        queue.write_buffer(&self.buffer, 0, new_content);
        self.content = new_content.to_vec();
    }

    pub fn binding_resource(&self) -> wgpu::BindingResource{
        self.buffer.as_entire_binding()
    }

}
