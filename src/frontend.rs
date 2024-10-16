use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
};

use eframe::{App, CreationContext};
use egui::{Color32, ColorImage, Context, ImageData, Label, TextureHandle, TextureOptions, Ui};
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

pub struct EmulatorConfig {
    pub sw1: u8,
    pub sw2: u8,
    pub screen_mode: ScreenMode
}

impl Default for EmulatorConfig {
    fn default() -> Self {
        Self {
            sw1: DD_ENABLE | RESERVED | MEM_64K | DISPLAY_MDA_80_25 | DRIVES_2,
            sw2: HIGH_NIBBLE | TOTAL_RAM_64,
            screen_mode: ScreenMode::MDA4025,
        }
    }
}

#[derive(Default)]
pub struct Application {
    instance: wgpu::Instance,
    state: Option<AppState>,
    window: Option<Arc<Window>>,

    config: EmulatorConfig
}

impl Application {
    pub fn new(config: EmulatorConfig) -> Self {
        let instance = egui_wgpu::wgpu::Instance::new(wgpu::InstanceDescriptor::default());

        Self {
            state: None,
            instance,
            window: None,

            config
        }
    }

    async fn set_window(&mut self, window: Window) {
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
            &self.config
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
        ];

        self.state.as_mut().unwrap().emulator.texture_handles = Some(texture_handles);
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();

        pollster::block_on(self.set_window(window));

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

#[derive(Default)]
struct Emulator {
    sys: Arc<Mutex<System>>,

    show_mda: bool,
    show_cga_40x25: bool,
    show_cga_80x25: bool,

    floppy_path: [Option<PathBuf>; 2],

    texture_handles: Option<[TextureHandle; 3]>,
}

impl Emulator {
    pub fn new(sw1: u8, sw2: u8, screen_mode: ScreenMode) -> Self {
        Self {
            sys: Arc::new(Mutex::new(System::new(sw1, sw2, screen_mode))),

            show_cga_40x25: false,
            show_cga_80x25: false,
            show_mda: false,

            floppy_path: [None, None],

            texture_handles: None,
        }
    }

    pub fn with_cfg(emulator_config: &EmulatorConfig) -> Self {
        Self::new(emulator_config.sw1, emulator_config.sw2, emulator_config.screen_mode)
    }

    fn add_screen_windows(&mut self, ctx: &Context) {
        let size = (720., 350. * 1.33);

        if self.show_mda {
            egui::Window::new("MDA")
                .open(&mut self.show_mda)
                .show(ctx, |ui| {
                    ui.set_min_size(size.into());

                    let frame = self.sys.lock().unwrap().create_mda_frame();

                    self.texture_handles.as_mut().unwrap()[0].set(
                        ColorImage::from_rgb([720, 350], &frame),
                        TextureOptions::NEAREST,
                    );

                    ui.add(
                        egui::Image::new(&self.texture_handles.as_ref().unwrap()[0])
                            .fit_to_exact_size(size.into())
                            .texture_options(TextureOptions::NEAREST)
                            .maintain_aspect_ratio(false),
                    );
                });
        }

        if self.show_cga_40x25 {
            let size = (320. * 2., 200. * 2. * 1.2);

            egui::Window::new("CGA 40x25")
                .open(&mut self.show_cga_40x25)
                .show(ctx, |ui| {
                    ui.set_min_size(size.into());

                    let frame = self.sys.lock().unwrap().create_cga_40x25_frame();

                    self.texture_handles.as_mut().unwrap()[1].set(
                        ColorImage::from_rgb([320, 200], &frame),
                        TextureOptions::NEAREST,
                    );

                    ui.add(
                        egui::Image::new(&self.texture_handles.as_ref().unwrap()[1])
                            .fit_to_exact_size(size.into())
                            .maintain_aspect_ratio(false),
                    )
                });
        }

        if self.show_cga_80x25 {
            let size = (640., 200. * 2.4);

            egui::Window::new("CGA 80x25")
                .open(&mut self.show_cga_80x25)
                .show(ctx, |ui| {
                    ui.set_min_size(size.into());

                    let frame = self.sys.lock().unwrap().create_cga_80x25_frame();

                    self.texture_handles.as_mut().unwrap()[2].set(
                        ColorImage::from_rgb([640, 200], &frame),
                        TextureOptions::NEAREST,
                    );

                    ui.add(
                        egui::Image::new(&self.texture_handles.as_ref().unwrap()[2])
                            .fit_to_exact_size(size.into())
                            .maintain_aspect_ratio(false),
                    )
                });
        }

        ctx.request_repaint();
    }

    fn add_menu(&mut self, ui: &mut Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Reset").clicked() {
                    self.sys.lock().unwrap().rst();
                    self.sys.lock().unwrap().load_roms();
                }

                if self.sys.lock().unwrap().running {
                    if ui.button("Stop").clicked() {
                        self.sys.lock().unwrap().running = false;
                    }
                } else if ui.button("Start").clicked() {
                    self.sys.lock().unwrap().running = true;
                }
            });

            ui.menu_button("Debug", |_ui| {});

            ui.menu_button("Screen", |ui| {
                ui.menu_button("MDA", |ui| {
                    if ui.button("80x25").clicked() {
                        self.show_mda = !self.show_mda;
                    }
                });

                ui.menu_button("CGA", |ui| {
                    if ui.button("40x25").clicked() {
                        self.show_cga_40x25 = !self.show_cga_40x25;
                    }

                    if ui.button("80x25").clicked() {
                        self.show_cga_80x25 = !self.show_cga_80x25;
                    }
                })
            });

            ui.menu_button("Floppys", |ui| {
                ui.menu_button("Floppy 1", |ui| {
                    self.load_floppy_button(ui, 0);
                });
                ui.menu_button("Floppy 2", |ui| {
                    self.load_floppy_button(ui, 1);
                });
            })
        });
    }

    fn load_floppy_button(&mut self, ui: &mut Ui, floppy: usize) {
        if self.floppy_path[floppy].is_none() {
            if ui.button("Open file...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.floppy_path[floppy] = Some(path.clone());

                    self.sys
                        .lock()
                        .unwrap()
                        .inser_floppy_disk(path.to_str().unwrap(), floppy);
                    // println!("{:?}", path.to_str());
                }
            }
        } else {
            ui.add(
                Label::new(format!(
                    "{:?}",
                    self.floppy_path[floppy]
                        .as_ref()
                        .unwrap()
                        .file_name()
                        .unwrap()
                ))
                .wrap_mode(egui::TextWrapMode::Extend),
            );

            // ui.label();
            if ui.button("Unmount").clicked() {
                self.floppy_path[floppy] = None;
            }
        }
    }
}

struct AppState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'static>,
    scale_factor: f32,
    egui_renderer: EguiRenderer,

    emulator: Emulator,
}

impl AppState {
    pub async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        window: &Window,
        width: u32,
        height: u32,
        emulator_config: &EmulatorConfig
    ) -> Self {
        let power_pref = wgpu::PowerPreference::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: power_pref,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let features = wgpu::Features::empty();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: features,
                    ..Default::default()
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let selected_format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let swapchain_format = swapchain_capabilities
            .formats
            .iter()
            .find(|d| **d == selected_format)
            .expect("failed to select proper surface texture format!");

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *swapchain_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 0,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);

        let egui_renderer = EguiRenderer::new(&device, surface_config.format, None, 1, window);

        let scale_factor = 1.0;

        Self {
            device,
            queue,
            surface,
            surface_config,
            scale_factor,
            egui_renderer,
            emulator: Emulator::with_cfg(emulator_config),
        }
    }

    fn resize_surface(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn run_emulation(&mut self) {
        let sys = self.emulator.sys.clone();

        std::thread::spawn(move || {
            const UPDATE_RATE_MS: u128 = 5;

            sys.lock().unwrap().rst();
            sys.lock().unwrap().load_roms();

            let mut start = Instant::now();

            loop {
                let mut end = Instant::now();
                let mut elapsed = end.duration_since(start);

                while elapsed.as_millis() < UPDATE_RATE_MS {
                    end = Instant::now();
                    elapsed = end.duration_since(start);
                }

                start = Instant::now();
                sys.lock().unwrap().update(elapsed.as_secs_f32());
            }
        });
    }
}

struct EguiRenderer {
    state: egui_winit::State,
    renderer: Renderer,
    frame_started: bool,
}

impl EguiRenderer {
    pub fn context(&self) -> &Context {
        self.state.egui_ctx()
    }

    pub fn new(
        device: &Device,
        output_color_format: TextureFormat,
        output_depth_format: Option<TextureFormat>,
        msaa_samples: u32,
        window: &Window,
    ) -> EguiRenderer {
        let egui_context = Context::default();

        let egui_state = egui_winit::State::new(
            egui_context,
            egui::viewport::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            Some(2 * 1024), // default dimension is 2048
        );
        let egui_renderer = Renderer::new(
            device,
            output_color_format,
            output_depth_format,
            msaa_samples,
            true,
        );

        EguiRenderer {
            state: egui_state,
            renderer: egui_renderer,
            frame_started: false,
        }
    }

    pub fn handle_input(&mut self, window: &Window, event: &WindowEvent) {
        let _ = self.state.on_window_event(window, event);
    }

    pub fn ppp(&mut self, v: f32) {
        self.context().set_pixels_per_point(v);
    }

    pub fn begin_frame(&mut self, window: &Window) {
        let raw_input = self.state.take_egui_input(window);
        self.state.egui_ctx().begin_pass(raw_input);
        self.frame_started = true;
    }

    pub fn end_frame_and_draw(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        window: &Window,
        window_surface_view: &TextureView,
        screen_descriptor: ScreenDescriptor,
    ) {
        if !self.frame_started {
            panic!("begin_frame must be called before end_frame_and_draw can be called!");
        }

        self.ppp(screen_descriptor.pixels_per_point);

        let full_output = self.state.egui_ctx().end_pass();

        self.state
            .handle_platform_output(window, full_output.platform_output);

        let tris = self
            .state
            .egui_ctx()
            .tessellate(full_output.shapes, self.state.egui_ctx().pixels_per_point());
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }
        self.renderer
            .update_buffers(device, queue, encoder, &tris, &screen_descriptor);
        let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: window_surface_view,
                resolve_target: None,
                ops: egui_wgpu::wgpu::Operations {
                    load: egui_wgpu::wgpu::LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            label: Some("egui main render pass"),
            occlusion_query_set: None,
        });

        self.renderer
            .render(&mut rpass.forget_lifetime(), &tris, &screen_descriptor);
        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }

        self.frame_started = false;
    }
}
