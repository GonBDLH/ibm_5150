use std::{
    cell::RefCell,
    collections::HashMap,
    num::NonZeroU32,
    rc::Rc,
    thread,
    time::{self, Instant},
};

use log::{debug, info, trace};
use softbuffer::{Context, Surface};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize, Size},
    event::{ElementState, KeyEvent, StartCause, WindowEvent},
    event_loop::ControlFlow,
    platform::scancode::PhysicalKeyExtScancode,
    window::{self, Window, WindowId},
};

use crate::hardware::sys::System;

pub const DESIRED_FPS: f32 = 50.;
const WAIT_TIME: time::Duration = time::Duration::from_millis(20);
const POLL_SLEEP_TIME: time::Duration = time::Duration::from_millis(5);

struct GraphicsContext {
    /// The global softbuffer context.
    context: Context<Rc<Window>>,

    /// The hash map of window IDs to surfaces.
    surface: Surface<Rc<Window>, Rc<Window>>,
}

#[derive(Default)]
pub struct IbmPc {
    pub sys: System,

    request_redraw: bool,
    wait_cancelled: bool,
    close_requested: bool,
    window: Option<Rc<Window>>,
    window_dimensions: (f32, f32),

    frametime: Option<Instant>,
    poll_cycles: i32,

    graphics_context: Option<GraphicsContext>,
}

impl IbmPc {
    pub fn new(sw1: u8, sw2: u8, dimensions: (f32, f32)) -> Self {
        Self {
            sys: System::new(sw1, sw2, dimensions),
            window_dimensions: dimensions,
            ..Default::default()
        }
    }

    fn fill_window(&mut self, img_buffer: Vec<u8>) {
        if let Some(ctx) = self.graphics_context.as_mut() {
            let (width, height) = {
                let size = self.window.as_ref().unwrap().inner_size();
                (size.width, size.height)
            };

            // println!("{} {}", width, height);
            // println!("{} {}", self.window_dimensions.0 as u32, self.window_dimensions.1 as u32);

            ctx.surface
                .resize(
                    NonZeroU32::new(self.window_dimensions.0 as u32).unwrap(),
                    NonZeroU32::new(self.window_dimensions.1 as u32).unwrap(),
                )
                .unwrap();

            let mut buffer = ctx.surface.buffer_mut().unwrap();
            for index in 0..(self.window_dimensions.0 * self.window_dimensions.1) as usize {
                // let y = index / width;
                // let x = index % width;
                let red = img_buffer[index * 3] as u32;
                let green = img_buffer[index * 3 + 1] as u32;
                let blue = img_buffer[index * 3 + 2] as u32;

                buffer[index] = blue | (green << 8) | (red << 16);
            }

            buffer.present().unwrap();
        }
    }
}

impl ApplicationHandler for IbmPc {
    fn new_events(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        if cause == StartCause::Poll {
            self.poll_cycles += 1;

            if self.poll_cycles == 4 {
                self.poll_cycles = 0;
                self.window.as_ref().unwrap().request_redraw();
            }

            let elapsed = self.frametime.unwrap_or(Instant::now()).elapsed();

            self.sys.update(elapsed.as_secs_f32());

            trace!("FRAMETIME: {}", elapsed.as_micros());
            self.frametime = Some(Instant::now());
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        debug!("{event:?}");

        match event {
            WindowEvent::CloseRequested => {
                self.close_requested = true;
            }
            WindowEvent::RedrawRequested => {
                let window = self.window.as_ref().unwrap();
                window.pre_present_notify();
                let frame = self.sys.create_frame();
                self.fill_window(frame);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: key,
                        state,
                        ..
                    },
                ..
            } => match state {
                ElementState::Pressed => {
                    self.sys.bus.ppi.key_down(key.to_scancode().unwrap() as u8);
                }
                ElementState::Released => {
                    self.sys.bus.ppi.key_up(key.to_scancode().unwrap() as u8);
                }
            },

            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        thread::sleep(POLL_SLEEP_TIME);
        event_loop.set_control_flow(ControlFlow::Poll);

        if self.close_requested {
            event_loop.exit();
        }
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("IBM 5150 Emulator")
            .with_inner_size(PhysicalSize::new(
                self.window_dimensions.0,
                self.window_dimensions.1,
            ))
            .with_resizable(false);
        self.window = Some(Rc::new(
            event_loop.create_window(window_attributes).unwrap(),
        ));

        let context = Context::new(self.window.as_ref().unwrap().clone()).unwrap();
        let surface = Surface::new(&context, self.window.as_ref().unwrap().clone()).unwrap();

        self.graphics_context = Some(GraphicsContext { context, surface });
    }
}
