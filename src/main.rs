mod cartridge;
mod cpu;
mod gfx;
mod ines_rom;
mod instructions;
mod machine;
mod memory_map;
mod ppu;

use std::path::Path;

use ines_rom::INESRom;
use machine::Machine;
use winit::{
  error::EventLoopError,
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::WindowBuilder,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::gfx::gfx_state::GfxState;

pub async fn run() -> Result<(), EventLoopError> {
  let rom = INESRom::from_file(&Path::new("smb.nes")).unwrap();
  println!("Using mapper ID {}", rom.mapper_id);
  let mut machine = Machine::from_rom(rom);
  machine.reset();

  let event_loop = EventLoop::new()?;
  let builder = WindowBuilder::new().with_title("Family Computer");
  #[cfg(wasm_platform)]
  let builder = {
    use winit::platform::web::WindowBuilderExtWebSys;
    builder.with_append(true)
  };
  let window = builder.build(&event_loop).unwrap();

  #[cfg(wasm_platform)]
  let log_list = wasm::insert_canvas_and_create_log_list(&window);

  let mut gfx_state = GfxState::new(window).await;
  let mut prev_time = std::time::Instant::now();

  event_loop.run(|event, target| {
    #[cfg(wasm_platform)]
    wasm::log_event(&log_list, &event);

    match event {
      Event::AboutToWait => {
        machine.step();
        if (std::time::Instant::now() - prev_time).as_secs_f32() > (1.0 / 60.0) {
          gfx_state.update();
        } else {
          target.set_control_flow(ControlFlow::Poll);
        }
      }
      Event::WindowEvent { window_id, event } if window_id == gfx_state.window().id() => {
        if !gfx_state.input(&event) {
          match event {
            WindowEvent::CloseRequested => {
              target.exit();
            }
            WindowEvent::Resized(physical_size) => {
              gfx_state.resize(physical_size);
            }
            WindowEvent::RedrawRequested => {
              let current_time = std::time::Instant::now();
              let delta_time = current_time - prev_time;
              prev_time = current_time;

              gfx_state.update();
              match gfx_state.render(delta_time) {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => gfx_state.resize(gfx_state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
              }
            }
            // WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
            //   // new_inner_size is &&mut so we have to dereference it twice
            //   gfx_state.resize(**new_inner_size);
            // }
            _ => {}
          }
        }
      }
      _ => {}
    }
  })
}

pub fn main() -> Result<(), EventLoopError> {
  pollster::block_on(run())
}

#[cfg(wasm_platform)]
mod wasm {
  use std::num::NonZeroU32;

  use softbuffer::{Surface, SurfaceExtWeb};
  use wasm_bindgen::prelude::*;
  use winit::{
    event::{Event, WindowEvent},
    window::Window,
  };

  #[wasm_bindgen(start)]
  pub fn run() {
    console_log::init_with_level(log::Level::Debug).expect("error initializing logger");

    #[allow(clippy::main_recursion)]
    let _ = super::main();
  }

  pub fn insert_canvas_and_create_log_list(window: &Window) -> web_sys::Element {
    use winit::platform::web::WindowExtWebSys;

    let canvas = window.canvas().unwrap();
    let mut surface = Surface::from_canvas(canvas.clone()).unwrap();
    surface
      .resize(
        NonZeroU32::new(canvas.width()).unwrap(),
        NonZeroU32::new(canvas.height()).unwrap(),
      )
      .unwrap();
    let mut buffer = surface.buffer_mut().unwrap();
    buffer.fill(0xFFF0000);
    buffer.present().unwrap();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    let style = &canvas.style();
    style.set_property("margin", "50px").unwrap();
    // Use to test interactions with border and padding.
    //style.set_property("border", "50px solid black").unwrap();
    //style.set_property("padding", "50px").unwrap();

    let log_header = document.create_element("h2").unwrap();
    log_header.set_text_content(Some("Event Log"));
    body.append_child(&log_header).unwrap();

    let log_list = document.create_element("ul").unwrap();
    body.append_child(&log_list).unwrap();
    log_list
  }

  pub fn log_event(log_list: &web_sys::Element, event: &Event<()>) {
    log::debug!("{:?}", event);

    // Getting access to browser logs requires a lot of setup on mobile devices.
    // So we implement this basic logging system into the page to give developers an easy alternative.
    // As a bonus its also kind of handy on desktop.
    let event = match event {
      Event::WindowEvent {
        event: WindowEvent::RedrawRequested,
        ..
      } => None,
      Event::WindowEvent { event, .. } => Some(format!("{event:?}")),
      Event::Resumed | Event::Suspended => Some(format!("{event:?}")),
      _ => None,
    };
    if let Some(event) = event {
      let window = web_sys::window().unwrap();
      let document = window.document().unwrap();
      let log = document.create_element("li").unwrap();

      let date = js_sys::Date::new_0();
      log.set_text_content(Some(&format!(
        "{:02}:{:02}:{:02}.{:03}: {event}",
        date.get_hours(),
        date.get_minutes(),
        date.get_seconds(),
        date.get_milliseconds(),
      )));

      log_list
        .insert_before(&log, log_list.first_child().as_ref())
        .unwrap();
    }
  }
}
