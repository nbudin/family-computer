use std::time::Duration;

use glyphon::{Resolution, TextBounds};
use winit::{event::WindowEvent, window::Window};

use super::{
  crt_screen::{CRTScreen, PIXEL_BUFFER_SIZE},
  text_controller::{LabelID, TextController},
};

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
  crt_screen: CRTScreen,
  text_controller: TextController,
  fps_label_id: LabelID,
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

    let crt_screen = CRTScreen::new(&device, config.format);
    let mut text_controller = TextController::new(&device, &queue, config.format);

    let fps_label_id = {
      text_controller.add_label(
        glyphon::Metrics {
          font_size: 2.0,
          line_height: 2.0,
        },
        TextBounds {
          top: 0,
          left: 0,
          right: size.width.try_into().unwrap(),
          bottom: size.height.try_into().unwrap(),
        },
        glyphon::Attrs::new()
          .family(glyphon::Family::Name("Pixel NES"))
          .color(glyphon::Color::rgb(255, 255, 255)),
        glyphon::Shaping::Basic,
      )
    };

    GfxState {
      surface,
      device,
      queue,
      config,
      size,
      window,
      crt_screen,
      text_controller,
      fps_label_id,
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

  pub fn input(&mut self, _event: &WindowEvent) -> bool {
    // TODO
    false
  }

  pub fn get_pixbuf_mut(&mut self) -> &mut [u8; PIXEL_BUFFER_SIZE] {
    &mut self.crt_screen.next_frame
  }

  pub fn update(&mut self, delta_time: Duration) {
    self.text_controller.set_label_text(
      self.fps_label_id,
      format!("{:.02} FPS", 1.0 / delta_time.as_secs_f32()).as_str(),
    );
  }

  pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
      self
        .crt_screen
        .prepare(&self.queue, self.window.inner_size());
      self.text_controller.prepare(
        &self.device,
        &self.queue,
        Resolution {
          width: self.size.width,
          height: self.size.height,
        },
      );

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

      self.crt_screen.render(&mut render_pass);
      self.text_controller.render(&mut render_pass).unwrap();
    }

    // submit will accept anything that implements IntoIter
    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
  }
}
