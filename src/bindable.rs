#[allow(unused)]
use anyhow::*;

pub trait BindGoupLayout{
    fn create_bind_group_layout(device: &wgpu::Device, label: Option<&str>) -> anyhow::Result<wgpu::BindGroupLayout>;
}

pub trait BindGroup: BindGoupLayout{
    fn create_bind_group(&self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout, label: Option<&str>) -> anyhow::Result<wgpu::BindGroup>;
}
