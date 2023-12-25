use std::mem::size_of;

use wgpu::util::DeviceExt;
use winit::dpi::{PhysicalPosition, PhysicalSize};

use super::{
  node::{BoxError, Node, RenderablePrepareData},
  vertex2d::Vertex2D,
};

pub const PIXEL_BUFFER_WIDTH: u32 = 256;
pub const PIXEL_BUFFER_HEIGHT: u32 = 240;
pub const BYTES_PER_PIXEL: u32 = 4;
pub const PIXEL_BUFFER_SIZE: usize = 256 * 240 * 4;
pub const PIXEL_BUFFER_ASPECT: f32 = 256.0 / 240.0;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WindowDimensionsUniform {
  width: u32,
  height: u32,
}

pub struct CRTScreen {
  pub render_pipeline: wgpu::RenderPipeline,
  pub texture: wgpu::Texture,
  pub view: wgpu::TextureView,
  pub sampler: wgpu::Sampler,
  pub bind_group_layout: wgpu::BindGroupLayout,
  pub bind_group: wgpu::BindGroup,
  pub next_frame: [u8; PIXEL_BUFFER_SIZE],
  pub position: PhysicalPosition<u32>,
  pub size: PhysicalSize<u32>,
  vertex_buffer: wgpu::Buffer,
  index_buffer: wgpu::Buffer,
  uniform_buffer: wgpu::Buffer,
  num_indices: u32,
}

impl CRTScreen {
  pub fn new(
    device: &wgpu::Device,
    target_texture_format: wgpu::TextureFormat,
    position: PhysicalPosition<u32>,
    size: PhysicalSize<u32>,
  ) -> Self {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
      size: wgpu::Extent3d {
        width: PIXEL_BUFFER_WIDTH.try_into().unwrap(),
        height: PIXEL_BUFFER_HEIGHT.try_into().unwrap(),
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba8UnormSrgb,
      usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
      label: Some("texture"),
      view_formats: &[],
    });

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      address_mode_u: wgpu::AddressMode::ClampToEdge,
      address_mode_v: wgpu::AddressMode::ClampToEdge,
      address_mode_w: wgpu::AddressMode::ClampToEdge,
      mag_filter: wgpu::FilterMode::Nearest,
      min_filter: wgpu::FilterMode::Nearest,
      mipmap_filter: wgpu::FilterMode::Nearest,
      ..Default::default()
    });

    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("Uniform Buffer"),
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      size: size_of::<WindowDimensionsUniform>().try_into().unwrap(),
      mapped_at_creation: false,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("bind_group_layout"),
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 2,
          visibility: wgpu::ShaderStages::VERTEX,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        },
      ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("bind_group"),
      layout: &bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: wgpu::BindingResource::TextureView(&view),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::Sampler(&sampler),
        },
        wgpu::BindGroupEntry {
          binding: 2,
          resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &uniform_buffer,
            offset: 0,
            size: None,
          }),
        },
      ],
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("Shader"),
      source: wgpu::ShaderSource::Wgsl(include_str!("crt_screen.wgsl").into()),
    });

    let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("Vertex Buffer"),
      usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
      size: u64::try_from(size_of::<Vertex2D>()).unwrap() * 4,
      mapped_at_creation: false,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Index Buffer"),
      contents: bytemuck::cast_slice(INDICES),
      usage: wgpu::BufferUsages::INDEX,
    });
    let num_indices: u32 = INDICES.len().try_into().unwrap();

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Render Pipeline Layout"),
      bind_group_layouts: &[&bind_group_layout],
      push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(&render_pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: "vs_main", // 1.
        buffers: &[Vertex2D::desc()],
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: "fs_main",
        targets: &[Some(wgpu::ColorTargetState {
          format: target_texture_format,
          blend: Some(wgpu::BlendState::REPLACE),
          write_mask: wgpu::ColorWrites::ALL,
        })],
      }),
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList, // 1.
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw, // 2.
        cull_mode: Some(wgpu::Face::Back),
        // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
        polygon_mode: wgpu::PolygonMode::Fill,
        // Requires Features::DEPTH_CLIP_CONTROL
        unclipped_depth: false,
        // Requires Features::CONSERVATIVE_RASTERIZATION
        conservative: false,
      },
      depth_stencil: None, // 1.
      multisample: wgpu::MultisampleState {
        count: 1,                         // 2.
        mask: !0,                         // 3.
        alpha_to_coverage_enabled: false, // 4.
      },
      multiview: None, // 5.
    });

    Self {
      texture,
      view,
      sampler,
      bind_group,
      bind_group_layout,
      render_pipeline,
      index_buffer,
      vertex_buffer,
      uniform_buffer,
      num_indices,
      next_frame: [0; PIXEL_BUFFER_SIZE],
      position,
      size,
    }
  }
}

impl Node for CRTScreen {
  fn prepare(&mut self, data: &RenderablePrepareData) -> Result<(), BoxError> {
    let queue = data.queue;
    let window_size = data.window.inner_size();

    queue.write_buffer(
      &self.uniform_buffer,
      0,
      bytemuck::bytes_of(&WindowDimensionsUniform {
        width: window_size.width,
        height: window_size.height,
      }),
    );

    queue.write_buffer(
      &self.vertex_buffer,
      0,
      bytemuck::cast_slice(&[
        // Top left
        Vertex2D {
          position: [self.position.x, self.position.y],
          tex_coords: [0.0, 0.0],
        },
        // Top right
        Vertex2D {
          position: [(self.position.x + self.size.width), self.position.y],
          tex_coords: [1.0, 0.0],
        },
        // Bottom left
        Vertex2D {
          position: [self.position.x, (self.position.y + self.size.height)],
          tex_coords: [0.0, 1.0],
        },
        // Bottom right
        Vertex2D {
          position: [
            (self.position.x + self.size.width),
            (self.position.y + self.size.height),
          ],
          tex_coords: [1.0, 1.0],
        },
      ]),
    );

    queue.write_texture(
      wgpu::ImageCopyTexture {
        texture: &self.texture,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
      },
      &self.next_frame,
      wgpu::ImageDataLayout {
        offset: 0,
        bytes_per_row: Some(BYTES_PER_PIXEL * PIXEL_BUFFER_WIDTH),
        rows_per_image: Some(PIXEL_BUFFER_HEIGHT),
      },
      self.texture.size(),
    );

    Ok(())
  }

  fn render<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) -> Result<(), BoxError> {
    pass.set_pipeline(&self.render_pipeline);
    pass.set_bind_group(0, &self.bind_group, &[]);
    pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
    pass.draw_indexed(0..self.num_indices, 0, 0..1);

    Ok(())
  }
}

const INDICES: &[u16] = &[0, 2, 1, 1, 2, 3];
