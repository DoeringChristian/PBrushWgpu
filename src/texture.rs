use image::GenericImageView;
use anyhow::*;
use crate::render_target::*;
use crate::bindable::*;

///
/// 
///
pub struct Texture{
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub format: wgpu::TextureFormat,
}

impl Texture{
    pub fn new_black(
        dim: (u32, u32),
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: Option<&str>,
        format: wgpu::TextureFormat
    ) -> Result<Self>{
        let data: Vec<u8> = vec![0; (dim.0 * dim.1 * 4) as usize];

        let size = wgpu::Extent3d{
            width: dim.0,
            height: dim.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(
            &wgpu::TextureDescriptor{
                label,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
            }
        );
        queue.write_texture(
            wgpu::ImageCopyTexture{
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &data,
            wgpu::ImageDataLayout{
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * dim.0),
                rows_per_image: std::num::NonZeroU32::new(dim.1),
            },
            size,
        );
        let texture_view_desc = wgpu::TextureViewDescriptor{
            format: Some(format),
            ..Default::default()
        };
        let view = texture.create_view(&texture_view_desc);
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor{
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }
        );

        Ok(Self{
            texture,
            view,
            sampler,
            format,
        })
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
        format: wgpu::TextureFormat
    ) -> Result<Self>{
        let img_data: &[u8] = match format{
            wgpu::TextureFormat::Rgba8Unorm => img.as_rgba8().unwrap(),
            wgpu::TextureFormat::Rgba8UnormSrgb => img.as_rgba8().unwrap(),
            wgpu::TextureFormat::Bgra8Unorm => img.as_bgra8().unwrap(),
            wgpu::TextureFormat::Bgra8UnormSrgb => img.as_bgra8().unwrap(),

            _ => {
                return Err(anyhow!("Format not supported"));
            }
        };
        let dims = img.dimensions();
        
        let size = wgpu::Extent3d{
            width: dims.0,
            height: dims.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(
            &wgpu::TextureDescriptor{
                label,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
            }
        );

        queue.write_texture(
            wgpu::ImageCopyTexture{
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            img_data,
            wgpu::ImageDataLayout{
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * dims.0),
                rows_per_image: std::num::NonZeroU32::new(dims.1),
            },
            size,
        );
        let texture_view_desc = wgpu::TextureViewDescriptor{
            format: Some(format),
            ..Default::default()
        };

        let view = texture.create_view(&texture_view_desc);
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor{
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }
        );

        Ok(Self{
            texture,
            view,
            sampler,
            format,
        })
    }

    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: Option<&str>,
        format: wgpu::TextureFormat
    ) -> Result<Self>{
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, label, format)
    }
}

impl RenderTarget for Texture{
    fn render_pass<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder, label: Option<&'a str>) -> Result<wgpu::RenderPass<'a>> {
        self.view.render_pass(encoder, label)
    }
}

impl BindGoupLayout for Texture{
    fn create_bind_group_layout(device: &wgpu::Device, label: Option<&str>) -> Result<wgpu::BindGroupLayout> {
        Ok(device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
            label,
            entries: &[
                wgpu::BindGroupLayoutEntry{
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Texture{
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float{filterable: true},
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry{
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        }))
    }
}

impl BindGroup for Texture{
    fn create_bind_group(&self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout, label: Option<&str>) -> anyhow::Result<wgpu::BindGroup> {
        Ok(device.create_bind_group(&wgpu::BindGroupDescriptor{
            label,
            layout,
            entries: &[
                wgpu::BindGroupEntry{
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.view),
                },
                wgpu::BindGroupEntry{
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                }
            ],
        }))
    }
}
