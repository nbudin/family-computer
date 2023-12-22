use std::time::Duration;

use bytemuck::{Pod, Zeroable};
use glyphon::{FontSystem, Resolution, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer};
use wgpu::{util::DeviceExt, MultisampleState};
use winit::{event::WindowEvent, window::Window};

pub struct GfxState {
  surface: wgpu::Surface,
  device: wgpu::Device,
  queue: wgpu::Queue,
  config: wgpu::SurfaceConfiguration,
  pub size: winit::dpi::PhysicalSize<u32>,
  // The window must be declared after the surface so
  // it gets dropped after it as the surface contains
  // unsafe references to the window's resources.
  window: Window,
  render_pipeline: wgpu::RenderPipeline,
  vertex_buffer: wgpu::Buffer,
  index_buffer: wgpu::Buffer,
  atlas: TextAtlas,
  text_renderer: TextRenderer,
  swash_cache: SwashCache,
  font_system: FontSystem,
  num_indices: u32,
}

impl GfxState {
  pub async fn new(window: Window) -> Self {
    let size = window.inner_size();

    // The instance is a handle to our GPU
    // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
      backends: wgpu::Backends::all(),
      ..Default::default()
    });

    // # Safety
    //
    // The surface needs to live as long as the window that created it.
    // State owns the window, so this should be safe.
    let surface = unsafe { instance.create_surface(&window) }.unwrap();

    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
      })
      .await
      .unwrap();

    let (device, queue) = adapter
      .request_device(
        &wgpu::DeviceDescriptor {
          features: wgpu::Features::empty(),
          // WebGL doesn't support all of wgpu's features, so if
          // we're building for the web, we'll have to disable some.
          limits: if cfg!(target_arch = "wasm32") {
            wgpu::Limits::downlevel_webgl2_defaults()
          } else {
            wgpu::Limits::default()
          },
          label: None,
        },
        None, // Trace path
      )
      .await
      .unwrap();

    let surface_caps = surface.get_capabilities(&adapter);
    // Shader code in this tutorial assumes an sRGB surface texture. Using a different
    // one will result in all the colors coming out darker. If you want to support non
    // sRGB surfaces, you'll need to account for that when drawing to the frame.
    let surface_format = surface_caps
      .formats
      .iter()
      .copied()
      .filter(|f| f.is_srgb())
      .next()
      .unwrap_or(surface_caps.formats[0]);
    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface_format,
      width: size.width,
      height: size.height,
      present_mode: surface_caps.present_modes[0],
      alpha_mode: surface_caps.alpha_modes[0],
      view_formats: vec![],
    };
    surface.configure(&device, &config);

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("Shader"),
      source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Render Pipeline Layout"),
      bind_group_layouts: &[],
      push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(&render_pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: "vs_main", // 1.
        buffers: &[Vertex::desc()],
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: "fs_main",
        targets: &[Some(wgpu::ColorTargetState {
          format: config.format,
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

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: bytemuck::cast_slice(VERTICES),
      usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Index Buffer"),
      contents: bytemuck::cast_slice(INDICES),
      usage: wgpu::BufferUsages::INDEX,
    });
    let num_indices: u32 = INDICES.len().try_into().unwrap();

    let mut atlas = TextAtlas::new(&device, &queue, wgpu::TextureFormat::Bgra8UnormSrgb);
    let text_renderer = TextRenderer::new(&mut atlas, &device, MultisampleState::default(), None);
    let swash_cache = SwashCache::new();
    let mut font_system = FontSystem::new();

    font_system
      .db_mut()
      .load_font_data(include_bytes!("Pixel_NES.otf").to_vec());

    GfxState {
      surface,
      device,
      queue,
      config,
      size,
      window,
      render_pipeline,
      vertex_buffer,
      index_buffer,
      num_indices,
      atlas,
      text_renderer,
      swash_cache,
      font_system,
    }
  }

  pub fn window(&self) -> &Window {
    &self.window
  }

  pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      self.size = new_size;
      self.config.width = new_size.width;
      self.config.height = new_size.height;
      self.surface.configure(&self.device, &self.config);
    }
  }

  pub fn input(&mut self, event: &WindowEvent) -> bool {
    // TODO
    false
  }

  pub fn update(&mut self) {
    self.window.request_redraw();
  }

  pub fn render(&mut self, delta_time: Duration) -> Result<(), wgpu::SurfaceError> {
    let output = self.surface.get_current_texture()?;
    let view = output
      .texture
      .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = self
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
      });

    {
      let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color {
              r: 0.1,
              g: 0.2,
              b: 0.3,
              a: 1.0,
            }),
            store: true,
          },
        })],
        depth_stencil_attachment: None,
      });

      render_pass.set_pipeline(&self.render_pipeline);
      render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
      render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16); // 1.
      render_pass.draw_indexed(0..self.num_indices, 0, 0..1);

      let mut text_buffer = glyphon::Buffer::new(
        &mut self.font_system,
        glyphon::Metrics {
          font_size: 2.0,
          line_height: 2.0,
        },
      );
      text_buffer.set_size(
        &mut self.font_system,
        self.size.width as f32,
        self.size.height as f32,
      );
      text_buffer.set_text(
        &mut self.font_system,
        format!("{} FPS", 1.0 / delta_time.as_secs_f32()).as_str(),
        glyphon::Attrs::new()
          .color(glyphon::Color::rgb(255, 255, 255))
          .family(glyphon::Family::Name("Pixel NES")),
        glyphon::Shaping::Basic,
      );
      let text_areas: Vec<TextArea> = vec![TextArea {
        left: 0.0,
        top: 0.0,
        scale: 16.0,
        bounds: TextBounds {
          top: 0,
          left: 0,
          right: self.size.width.try_into().unwrap(),
          bottom: self.size.height.try_into().unwrap(),
        },
        default_color: glyphon::Color::rgb(255, 255, 255),
        buffer: &text_buffer,
      }];
      self
        .text_renderer
        .prepare(
          &self.device,
          &self.queue,
          &mut self.font_system,
          &mut self.atlas,
          Resolution {
            height: self.size.height,
            width: self.size.width,
          },
          text_areas,
          &mut self.swash_cache,
        )
        .unwrap();

      self
        .text_renderer
        .render(&self.atlas, &mut render_pass)
        .unwrap();
    }

    // submit will accept anything that implements IntoIter
    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
  }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
  position: [f32; 3],
  color: [f32; 3],
}

impl Vertex {
  pub fn desc() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &[
        wgpu::VertexAttribute {
          offset: 0,
          shader_location: 0,
          format: wgpu::VertexFormat::Float32x3,
        },
        wgpu::VertexAttribute {
          offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
          shader_location: 1,
          format: wgpu::VertexFormat::Float32x3,
        },
      ],
    }
  }
}

const VERTICES: &[Vertex] = &[
  Vertex {
    position: [-0.0868241, 0.49240386, 0.0],
    color: [0.5, 0.0, 0.5],
  }, // A
  Vertex {
    position: [-0.49513406, 0.06958647, 0.0],
    color: [0.5, 0.0, 0.5],
  }, // B
  Vertex {
    position: [-0.21918549, -0.44939706, 0.0],
    color: [0.5, 0.0, 0.5],
  }, // C
  Vertex {
    position: [0.35966998, -0.3473291, 0.0],
    color: [0.5, 0.0, 0.5],
  }, // D
  Vertex {
    position: [0.44147372, 0.2347359, 0.0],
    color: [0.5, 0.0, 0.5],
  }, // E
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];
