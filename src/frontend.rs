use std::{
    cell::RefCell,
    collections::HashMap,
    default,
    num::NonZeroU32,
    rc::Rc,
    thread,
    time::{self, Instant},
};

use egui::emath::Float;
use log::{debug, info, trace};
use rayon::{
    iter::{IndexedParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};
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

const FRAMETIME_MS: u64 = 20;

// #[cfg(not(debug_assertions))]
const UPDATE_RATE_MS: u64 = 5;

// #[cfg(debug_assertions)]
// const UPDATE_RATE_MS: u64 = 19;

const POLL_SLEEP_TIME: time::Duration = time::Duration::from_millis(UPDATE_RATE_MS);

struct GraphicsContext {
    /// The global softbuffer context.
    context: Context<Rc<Window>>,

    /// The hash map of window IDs to surfaces.
    surface: Surface<Rc<Window>, Rc<Window>>,
}

#[derive(Clone, Copy, PartialEq)]
pub struct ScreenMode {
    dimensions: (f32, f32),
    aspect_ratio: f32,
}

impl ScreenMode {
    pub const MDA4025: ScreenMode = ScreenMode {
        dimensions: (720., 350.),
        aspect_ratio: 1.333,
    };

    pub const CGA4025: ScreenMode = ScreenMode {
        dimensions: (320., 200.),
        aspect_ratio: 1.2,
    };

    pub const CGA8025: ScreenMode = ScreenMode {
        dimensions: (640., 200.),
        aspect_ratio: 2.4,
    };
}

impl Default for ScreenMode {
    fn default() -> Self {
        ScreenMode::MDA4025
    }
}

impl ScreenMode {
    pub fn get_pixel_dimensions(&self) -> (f32, f32) {
        self.dimensions
    }

    pub fn get_aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }
}

#[derive(Default)]
pub struct IbmPc {
    pub sys: System,

    request_redraw: bool,
    wait_cancelled: bool,
    close_requested: bool,
    window: Option<Rc<Window>>,
    screen_mode: ScreenMode,

    updatetime: Option<Instant>,
    frametime: f32,
    poll_cycles: u64,

    graphics_context: Option<GraphicsContext>,
}

impl IbmPc {
    pub fn new(sw1: u8, sw2: u8, screen_mode: ScreenMode) -> Self {
        Self {
            sys: System::new(sw1, sw2, screen_mode),
            screen_mode,
            ..Default::default()
        }
    }

    fn draw_screen(&mut self, img_buffer: Vec<u8>) {
        if let Some(ctx) = self.graphics_context.as_mut() {
            let (width, height) = {
                let size = self.window.as_ref().unwrap().inner_size();
                (size.width, size.height)
            };

            ctx.surface
                .resize(
                    NonZeroU32::new(width).unwrap(),
                    NonZeroU32::new(height).unwrap(),
                )
                .unwrap();

            let mut buffer = ctx.surface.buffer_mut().unwrap();

            buffer
                .par_chunks_mut(width as usize)
                .enumerate()
                .for_each(|(y, row)| {
                    for x in 0..width {
                        let src_x = ((x as f32) / (width as f32)
                            * self.screen_mode.get_pixel_dimensions().0)
                            .round() as usize;
                        let src_y = ((y as f32) / (height as f32)
                            * self.screen_mode.get_pixel_dimensions().1)
                            .round() as usize;
                        let src_x =
                            src_x.min((self.screen_mode.get_pixel_dimensions().0 - 1.) as usize);
                        let src_y =
                            src_y.min((self.screen_mode.get_pixel_dimensions().1 - 1.) as usize);

                        let src_index =
                            (src_y * self.screen_mode.get_pixel_dimensions().0 as usize + src_x)
                                * 3;

                        let red = img_buffer[src_index] as u32;
                        let green = img_buffer[src_index + 1] as u32;
                        let blue = img_buffer[src_index + 2] as u32;

                        let src_color = (red << 16) | (green << 8) | blue;

                        row[x as usize] = src_color;
                    }
                });

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
            if self.frametime >= 0.019 {
                self.frametime -= 0.019;
                self.window.as_ref().unwrap().request_redraw();
            }

            let elapsed = self
                .updatetime
                .unwrap_or(Instant::now())
                .elapsed()
                .as_secs_f32();
            self.frametime += elapsed;

            trace!("{}", elapsed);

            self.sys.update(elapsed);
            self.updatetime = Some(Instant::now());
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
                self.draw_screen(frame);
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
                self.screen_mode.get_pixel_dimensions().0 * 1.5,
                self.screen_mode.get_pixel_dimensions().1
                    * 1.5
                    * self.screen_mode.get_aspect_ratio(),
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
