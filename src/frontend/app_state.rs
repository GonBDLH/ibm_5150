use std::default;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::egui_renderer::EguiRenderer;
use super::hardware::sys::{ScreenMode, System};
use super::*;

use egui::{RadioButton, SelectableLabel, TextureHandle};
use hardware::sys::BiosSelected;
use winit::event::WindowEvent;
use winit::window::Window;

// NEGADO PORQUE SW CONFIG -> OFF = 1, ON = 0

pub fn u8_to_bool_array(val: u8) -> [bool; 8] {
    let mut ret = [false; 8];

    #[allow(clippy::needless_range_loop)]
    for i in 0..8 {
        ret[i] = (!val >> (7 - i)) & 1 > 0;
    }

    ret
}

pub fn bool_array_to_u8(val: &[bool; 8]) -> u8 {
    let mut ret = 0;

    for (i, value) in val.iter().enumerate() {
        ret |= (!*value as u8) << (7 - i);
    }

    ret
}

#[derive(Default)]
pub struct EmulatorGui {
    pub sys: Arc<Mutex<System>>,

    show_mda: bool,
    show_cga_40x25: bool,
    show_cga_80x25: bool,

    show_cfg_window: bool,

    bios_selected: BiosSelected,

    pub sw1: [bool; 8],
    pub sw2: [bool; 8],

    floppy_path: [Option<PathBuf>; 2],

    pub texture_handles: Option<[TextureHandle; 3]>,
}

impl EmulatorGui {
    fn new(sw1: u8, sw2: u8) -> Self {
        Self {
            sys: Arc::new(Mutex::new(System::new(sw1, sw2))),

            show_cga_40x25: false,
            show_cga_80x25: false,
            show_mda: false,

            show_cfg_window: false,

            bios_selected: BiosSelected::default(),

            sw1: u8_to_bool_array(sw1),
            sw2: u8_to_bool_array(sw2),

            floppy_path: [None, None],

            texture_handles: None,
        }
    }

    pub fn with_cfg(emulator_config: EmulatorConfig) -> Self {
        Self::new(emulator_config.sw1, emulator_config.sw2)
    }

    pub fn add_screen_windows(&mut self, ctx: &Context) {
        // MDA
        let size = (720. * 1.07, 350. * 1.33 * 1.07);

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

        // CGA 40x25
        let size = (320. * 2.4, 200. * 2.4 * 1.2);

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

        // CGA 80x25
        let size = (640. * 1.2, 200. * 2.4 * 1.2);

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

        // CONFIG
        egui::Window::new("Config")
            .open(&mut self.show_cfg_window)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.heading("SW 1");

                    ui.columns(8, |columns| {
                        for (i, ui) in columns.iter_mut().enumerate() {
                            ui.add(toggle_switch::toggle(&mut self.sw1[i]));
                        }
                    })
                });

                ui.separator();
                ui.vertical_centered(|ui| {
                    ui.heading("SW 2");

                    ui.columns(8, |columns| {
                        for (i, ui) in columns.iter_mut().enumerate() {
                            ui.add(toggle_switch::toggle(&mut self.sw2[i]));
                        }
                    })
                });

                ui.separator();

                // let floppys_enabled = bool_array_to_u8(&self.sw1) & DD_ENABLE > 0;
                // let number_drives = ((bool_array_to_u8(&self.sw1) & DRIVES_4) >> 6) + 1;
                // ui.label(format!("Number of drives: {}", number_drives * (floppys_enabled as u8)));
                // ui.end_row();

                // let enabled_8087 = bool_array_to_u8(&self.sw1) & 0b00000010 > 0;
                // ui.label(format!("8087 installed: {}", enabled_8087));
                // ui.end_row();

                // let installed_base_ram = bool_array_to_u8(&self.sw1) & MEM_64K;
                // let text = match installed_base_ram {
                //     MEM_16K => "Base RAM: 16KB",
                //     MEM_32K => "Base RAM: 32KB",
                //     MEM_48K => "Base RAM: 48KB",
                //     MEM_64K => "Base RAM: 64KB",
                //     _ => unreachable!()
                // };
                // ui.label(text);
                // ui.end_row();

                // let screen = bool_array_to_u8(&self.sw1) & DISPLAY_MDA_80_25;
                // let text = match screen {
                //     DISPLAY_CGA_40_25 => "Display: CGA 40x25",
                //     DISPLAY_CGA_80_25 => "Display: CGA 80x25",
                //     DISPLAY_MDA_80_25 => "Display: MDA 80x25",
                //     DISPLAY_RESERVED =>  "Display: Reserved",
                //     _ => unreachable!()
                // };
                // ui.label(text);
                // ui.end_row();

                // let total_ram = bool_array_to_u8(&self.sw2) & 0b00011111;
                // let text = match total_ram {
                //     TOTAL_RAM_0 => "EXTRA RAM: 0 KB",
                //     TOTAL_RAM_96 => "EXTRA RAM: 16 KB",
                //     TOTAL_RAM_128 => "EXTRA RAM: 32 KB",
                //     TOTAL_RAM_160 => "EXTRA RAM: 160 KB",
                //     TOTAL_RAM_192 => "EXTRA RAM: 192 KB",
                //     TOTAL_RAM_224 => "EXTRA RAM: 244 KB",
                //     TOTAL_RAM_256 => "EXTRA RAM: 256 KB",
                //     TOTAL_RAM_288 => "EXTRA RAM: 288 KB",
                //     TOTAL_RAM_320 => "EXTRA RAM: 320 KB",
                //     TOTAL_RAM_352 => "EXTRA RAM: 352 KB",
                //     TOTAL_RAM_384 => "EXTRA RAM: 384 KB",
                //     TOTAL_RAM_416 => "EXTRA RAM: 416 KB",
                //     TOTAL_RAM_448 => "EXTRA RAM: 448 KB",
                //     TOTAL_RAM_480 => "EXTRA RAM: 480 KB",
                //     TOTAL_RAM_512 => "EXTRA RAM: 512 KB",
                //     TOTAL_RAM_544 => "EXTRA RAM: 544 KB",
                //     TOTAL_RAM_576 => "EXTRA RAM: 576 KB",
                //     TOTAL_RAM_608 => "EXTRA RAM: 608 KB",
                //     TOTAL_RAM_640 => "EXTRA RAM: 640 KB",
                //     _ => "EXTRA RAM: Invalido"
                // };
                // ui.label(text);
                // ui.end_row();

                let ui_builder = egui::UiBuilder::new();

                ui.scope_builder(ui_builder, |ui| {
                    egui::Grid::new("my_grid")
                    .num_columns(2)
                    .spacing([100.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("TEST");
                        ui.label("TEST");
                        ui.end_row();
                        ui.label("TEST");
                        ui.label("TEST");
                        ui.end_row();
                        ui.label("TEST");
                        ui.label("TEST");
                        ui.end_row();
                        ui.label("TEST");
                        ui.label("TEST");
                        ui.end_row();
                        ui.label("TEST");
                        ui.label("TEST");
                        ui.end_row();
                        ui.label("TEST");
                        ui.label("TEST");
                        ui.end_row();
                        ui.label("TEST");
                        ui.label("TEST");
                        ui.end_row();
                        ui.label("TEST");
                        ui.label("TEST");
                        ui.end_row();
                        ui.label("TEST");
                        ui.label("TEST");
                        ui.end_row();
                        ui.label("TEST");
                        ui.label("TEST");
                        ui.end_row();
                        ui.label("TEST");
                        ui.label("TEST");
                        ui.end_row();
                        ui.label("TEST");
                        ui.label("TEST");
                        ui.end_row();
                    });
                });
                
            });
    }

    pub fn add_menu(&mut self, ui: &mut Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Reset").clicked() {
                    self.sys
                        .lock()
                        .unwrap()
                        .update_sw(bool_array_to_u8(&self.sw1), bool_array_to_u8(&self.sw2));
                    self.sys.lock().unwrap().rst();
                    self.sys.lock().unwrap().load_roms(&self.bios_selected);
                }

                if self.sys.lock().unwrap().running {
                    if ui.button("Stop").clicked() {
                        self.sys.lock().unwrap().running = false;
                    }
                } else if ui.button("Start").clicked() {
                    self.sys.lock().unwrap().running = true;
                }

                ui.separator();

                ui.menu_button("BIOS", |ui| {
                    ui.radio_value(&mut self.bios_selected, BiosSelected::IbmPc, "IBM PC BIOS");
                    ui.radio_value(&mut self.bios_selected, BiosSelected::GlaBios, "GLaBIOS");
                });

                if ui.button("Config").clicked() {
                    self.show_cfg_window = !self.show_cfg_window;
                };
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

    pub fn load_floppy_button(&mut self, ui: &mut Ui, floppy: usize) {
        if self.floppy_path[floppy].is_none() {
            if ui.button("Open file...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.floppy_path[floppy] = Some(path.clone());

                    self.sys
                        .lock()
                        .unwrap()
                        .inser_floppy_disk(path.to_str().unwrap(), floppy);
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
            if ui.button("Eject").clicked() {
                self.floppy_path[floppy] = None;
                self.sys.lock().unwrap().eject_floppy_disk(floppy);
            }
        }
    }
}

enum MemoryConfig {
    RAM_16,
    RAM_32,
    RAM_64,
    RAM_96,
    RAM_128,
    RAM_160,
    RAM_192,
    RAM_224,
    RAM_256,
    RAM_288,
    RAM_320,
    RAM_352,
    RAM_384,
    RAM_416,
    RAM_448,
    RAM_480,
    RAM_512,
    RAM_544,
    RAM_576,
    RAM_608,
    RAM_640,
}

pub struct EmulatorConfig {
    sw1: u8,
    sw2: u8,
    screen_mode: ScreenMode,
}

impl Default for EmulatorConfig {
    fn default() -> Self {
        Self {
            sw1: DD_ENABLE | RESERVED | MEM_64K | DISPLAY_MDA_80_25 | DRIVES_2,
            sw2: HIGH_NIBBLE | TOTAL_RAM_64,
            screen_mode: ScreenMode::MDA8025,
        }
    }
}

impl EmulatorConfig {
    pub fn builder() -> EmulatorConfig {
        Self {
            sw1: 0,
            sw2: HIGH_NIBBLE,
            screen_mode: ScreenMode::MDA8025,
        }
    }

    pub fn set_conventional_ram(mut self, conventional_ram: u8) -> Self {
        self.sw1 |= conventional_ram;
        self
    }

    pub fn enable_disk_drives(mut self) -> Self {
        self.sw1 |= DD_ENABLE;
        self
    }

    pub fn disable_disk_drives(mut self) -> Self {
        self.sw1 |= DD_DISABLE;
        self
    }

    pub fn set_disk_drives_number(mut self, num_drives: u8) -> Self {
        self.sw1 |= num_drives;
        self
    }

    pub fn set_total_ram(mut self, total_ram: u8) -> Self {
        self.sw2 |= total_ram;
        self
    }

    pub fn set_screen_mode(mut self, screen_mode: ScreenMode) -> Self {
        let sw1 = match screen_mode {
            ScreenMode::MDA8025 => Some(DISPLAY_MDA_80_25),

            ScreenMode::CGA4025 => Some(DISPLAY_CGA_40_25),

            ScreenMode::CGA8025 => Some(DISPLAY_CGA_80_25),

            _ => None,
        };

        if let Some(sw1_display) = sw1 {
            self.screen_mode = screen_mode;
            self.sw1 |= sw1_display;
        }

        self
    }
}

pub struct AppState {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'static>,
    pub scale_factor: f32,
    pub egui_renderer: EguiRenderer,

    pub emulator: EmulatorGui,
}

impl AppState {
    pub async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        window: &Window,
        width: u32,
        height: u32,
        emulator_config: EmulatorConfig,
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
            emulator: EmulatorGui::with_cfg(emulator_config),
        }
    }

    pub fn resize_surface(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn run_emulation(&mut self) {
        let sys = self.emulator.sys.clone();
        let bios_selected = self.emulator.bios_selected.clone();

        std::thread::spawn(move || {
            const UPDATE_RATE_MS: u128 = 5;

            sys.lock().unwrap().rst();
            sys.lock().unwrap().load_roms(&bios_selected);

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
