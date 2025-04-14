use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
};

use app_state::{AppState, EmulatorConfig};
use eframe::{App, CreationContext};
use egui::{Color32, ColorImage, Context, ImageData, Label, TextureHandle, TextureOptions, Ui};
use egui_renderer::EguiRenderer;
use egui_wgpu::wgpu::{CommandEncoder, Device, Queue, StoreOp, TextureFormat, TextureView};
use egui_wgpu::{Renderer, ScreenDescriptor};
use hardware::sys::ScreenMode;
use wgpu::core::{device, instance};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    platform::scancode::PhysicalKeyExtScancode,
    window::{Window, WindowId},
};

use crate::{hardware::sys::System, *};

pub mod app_state;
mod egui_renderer;
mod toggle_switch;

#[derive(Default)]
pub struct Application {
    instance: wgpu::Instance,
    state: Option<AppState>,
    window: Option<Arc<Window>>,
}

impl Application {
    pub fn new() -> Self {
        let instance = egui_wgpu::wgpu::Instance::new(wgpu::InstanceDescriptor::default());

        Self {
            state: None,
            instance,
            window: None,
        }
    }

    async fn set_window(&mut self, window: Window, emulator_config: EmulatorConfig) {
        let window = Arc::new(window);
        let initial_width = 1360;
        let initial_height = 768;

        let _ = window.request_inner_size(PhysicalSize::new(initial_width, initial_height));

        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("Failed to create surface!");

        let state = AppState::new(
            &self.instance,
            surface,
            &window,
            initial_width,
            initial_height,
            emulator_config,
        )
        .await;

        self.window.get_or_insert(window);
        self.state.get_or_insert(state);
    }

    fn handle_resized(&mut self, width: u32, height: u32) {
        self.state.as_mut().unwrap().resize_surface(width, height);
    }

    fn handle_redraw(&mut self) {
        let state = self.state.as_mut().unwrap();

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [state.surface_config.width, state.surface_config.height],
            pixels_per_point: self.window.as_ref().unwrap().scale_factor() as f32
                * state.scale_factor,
        };

        let surface_texture = state
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let window = self.window.as_ref().unwrap();

        {
            state.egui_renderer.begin_frame(window);

            egui::TopBottomPanel::top("top_pannel").show(state.egui_renderer.context(), |ui| {
                state.emulator.add_menu(ui);
            });

            egui::CentralPanel::default().show(state.egui_renderer.context(), |_ui| {
                state
                    .emulator
                    .add_screen_windows(state.egui_renderer.context());
            });

            state.egui_renderer.end_frame_and_draw(
                &state.device,
                &state.queue,
                &mut encoder,
                window,
                &surface_view,
                screen_descriptor,
            );
        }

        state.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }

    pub fn set_texture_handles(&mut self) {
        let texture_handles = [
            self.state
                .as_ref()
                .unwrap()
                .egui_renderer
                .state
                .egui_ctx()
                .load_texture(
                    "mda_40x25",
                    ImageData::Color(Arc::new(ColorImage::new([720, 350], Color32::BLACK))),
                    TextureOptions::NEAREST,
                ),
            self.state
                .as_ref()
                .unwrap()
                .egui_renderer
                .state
                .egui_ctx()
                .load_texture(
                    "cga_40x25",
                    ImageData::Color(Arc::new(ColorImage::new([320, 200], Color32::BLACK))),
                    TextureOptions::NEAREST,
                ),
            self.state
                .as_ref()
                .unwrap()
                .egui_renderer
                .state
                .egui_ctx()
                .load_texture(
                    "cga_80x25",
                    ImageData::Color(Arc::new(ColorImage::new([640, 200], Color32::BLACK))),
                    TextureOptions::NEAREST,
                ),
            self.state
                .as_ref()
                .unwrap()
                .egui_renderer
                .state
                .egui_ctx()
                .load_texture(
                    "ega_320x200_16",
                    ImageData::Color(Arc::new(ColorImage::new([320, 200], Color32::BLACK))),
                    TextureOptions::NEAREST
                ),
            self.state
                .as_ref()
                .unwrap()
                .egui_renderer
                .state
                .egui_ctx()
                .load_texture(
                    "ega_640x350_16",
                    ImageData::Color(Arc::new(ColorImage::new([640, 350], Color32::BLACK))),
                    TextureOptions::NEAREST
                ),
        ];

        self.state.as_mut().unwrap().emulator.texture_handles = Some(texture_handles);
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();

        pollster::block_on(self.set_window(window, EmulatorConfig::default()));

        self.set_texture_handles();

        self.state.as_mut().unwrap().run_emulation();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        // let egui render to process the event first
        self.state
            .as_mut()
            .unwrap()
            .egui_renderer
            .handle_input(self.window.as_ref().unwrap(), &event);

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.handle_redraw();

                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Resized(new_size) => {
                self.handle_resized(new_size.width, new_size.height);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key,
                        state,
                        ..
                    },
                ..
            } => match state {
                ElementState::Pressed => {
                    self.state
                        .as_mut()
                        .unwrap()
                        .emulator
                        .sys
                        .lock()
                        .unwrap()
                        .key_down(physical_key.to_scancode().unwrap() as u8);
                }
                ElementState::Released => {
                    self.state
                        .as_mut()
                        .unwrap()
                        .emulator
                        .sys
                        .lock()
                        .unwrap()
                        .key_up(physical_key.to_scancode().unwrap() as u8);
                }
            },
            _ => (),
        }
    }
}
