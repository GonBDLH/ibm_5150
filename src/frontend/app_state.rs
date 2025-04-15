use std::default;
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::hardware::sys::ScreenModeVariant;

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
        ret[i] = (val >> i) & 1 > 0;
    }

    ret
}

pub fn bool_array_to_u8(val: &[bool; 8]) -> u8 {
    let mut ret = 0;

    for (i, value) in val.iter().enumerate() {
        ret |= (*value as u8) << i;
    }

    ret
}

pub struct EmulatorGui {
    pub sys: Arc<Mutex<System>>,

    show_mda: bool,
    show_cga_40x25: bool,
    show_cga_80x25: bool,
    show_ega_320x250_16: bool,
    show_ega_640x350_16: bool,

    show_cfg_window: bool,

    bios_selected: BiosSelected,

    pub sw1: [bool; 8],
    pub sw2: [bool; 8],

    memory_cfg_gui: MemoryConfig,
    dd_num_cfg_gui: DDNumConfig,
    video_card_type: VideoCardTypeConfig,

    floppy_path: [Option<PathBuf>; 2],

    pub texture_handles: Option<[TextureHandle; 5]>,
}

impl EmulatorGui {
    fn new(sw1: u8, sw2: u8) -> Self {
        Self {
            sys: Arc::new(Mutex::new(System::new(sw1, sw2))),

            show_cga_40x25: false,
            show_cga_80x25: false,
            show_ega_320x250_16: false,
            show_ega_640x350_16: false,
            show_mda: false,

            show_cfg_window: false,

            bios_selected: BiosSelected::default(),

            sw1: u8_to_bool_array(sw1),
            sw2: u8_to_bool_array(sw2),

            memory_cfg_gui: MemoryConfig::from_switches(sw1, sw2),
            dd_num_cfg_gui: DDNumConfig::from_switches(sw1),
            video_card_type: VideoCardTypeConfig::from_switches(sw1),

            floppy_path: [None, None],

            texture_handles: None,
        }
    }

    pub fn with_cfg(emulator_config: EmulatorConfig) -> Self {
        Self::new(emulator_config.sw1, emulator_config.sw2)
    }

    pub fn add_screen_windows(&mut self, ctx: &Context) {
        // MDA
        let size = (720. * 1.25, 350. * 1.33 * 1.25);

        egui::Window::new("MDA")
            .open(&mut self.show_mda)
            .show(ctx, |ui| {
                ui.set_min_size(size.into());

                let frame = self.sys.lock().unwrap().create_mda_frame();

                self.texture_handles.as_mut().unwrap()[0].set(
                    ColorImage::from_rgb([720, 350], &frame),
                    TextureOptions::NEAREST.with_mipmap_mode(Some(egui::TextureFilter::Nearest)),
                );

                ui.add(
                    egui::Image::new(&self.texture_handles.as_ref().unwrap()[0])
                        .fit_to_exact_size(size.into())
                        .texture_options(TextureOptions::NEAREST.with_mipmap_mode(Some(egui::TextureFilter::Nearest)))
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

        // EGA 320x200_16 
        let size = (320. * 2.4, 200. * 2.4 * 1.2);

        egui::Window::new("EGA 320x200_16")
            .open(&mut self.show_ega_320x250_16)
            .show(ctx, |ui| {
                ui.set_min_size(size.into());

                let frame = self.sys.lock().unwrap().create_ega_320x200_16_frame();

                self.texture_handles.as_mut().unwrap()[2].set(
                    ColorImage::from_rgb([320, 200], &frame),
                    TextureOptions::NEAREST,
                );

                ui.add(
                    egui::Image::new(&self.texture_handles.as_ref().unwrap()[2])
                        .fit_to_exact_size(size.into())
                        .maintain_aspect_ratio(false),
                )
            });

        // EGA 640x350_16 
        let size = (640. * 1.2, 350. * 1.2 * 1.37);

        egui::Window::new("EGA 640x350_16")
            .open(&mut self.show_ega_640x350_16)
            .show(ctx, |ui| {
                ui.set_min_size(size.into());

                let frame = self.sys.lock().unwrap().create_ega_640x350_16_frame();

                self.texture_handles.as_mut().unwrap()[2].set(
                    ColorImage::from_rgb([640, 350], &frame),
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
                let ui_builder = egui::UiBuilder::new();

                ui.scope_builder(ui_builder, |ui| {
                    egui::Grid::new("my_grid")
                    .num_columns(2)
                    .spacing([100.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Memory:");
                        egui::ComboBox::from_id_salt("select_ram")
                            .selected_text(format!("{:?}", self.memory_cfg_gui))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.memory_cfg_gui, MemoryConfig::Ram16, "16 KB");
                                ui.selectable_value(&mut self.memory_cfg_gui, MemoryConfig::Ram32, "32 KB");
                                ui.selectable_value(&mut self.memory_cfg_gui, MemoryConfig::Ram48, "48 KB");
                                ui.selectable_value(&mut self.memory_cfg_gui, MemoryConfig::Ram64, "64 KB");
                                ui.selectable_value(&mut self.memory_cfg_gui, MemoryConfig::Ram96, "96 KB");
                                ui.selectable_value(&mut self.memory_cfg_gui, MemoryConfig::Ram128, "128 KB");
                                ui.selectable_value(&mut self.memory_cfg_gui, MemoryConfig::Ram160, "160 KB");
                                ui.selectable_value(&mut self.memory_cfg_gui, MemoryConfig::Ram192, "192 KB");
                                ui.selectable_value(&mut self.memory_cfg_gui, MemoryConfig::Ram224, "224 KB");
                                ui.selectable_value(&mut self.memory_cfg_gui, MemoryConfig::Ram256, "256 KB");
                                ui.selectable_value(&mut self.memory_cfg_gui, MemoryConfig::Ram288, "288 KB");
                                ui.selectable_value(&mut self.memory_cfg_gui, MemoryConfig::Ram320, "320 KB");
                                ui.selectable_value(&mut self.memory_cfg_gui, MemoryConfig::Ram352, "352 KB");
                                ui.selectable_value(&mut self.memory_cfg_gui, MemoryConfig::Ram384, "384 KB");
                                ui.selectable_value(&mut self.memory_cfg_gui, MemoryConfig::Ram416, "416 KB");
                                ui.selectable_value(&mut self.memory_cfg_gui, MemoryConfig::Ram448, "448 KB");
                                ui.selectable_value(&mut self.memory_cfg_gui, MemoryConfig::Ram480, "480 KB");
                                ui.selectable_value(&mut self.memory_cfg_gui, MemoryConfig::Ram512, "512 KB");
                                ui.selectable_value(&mut self.memory_cfg_gui, MemoryConfig::Ram544, "544 KB");
                                ui.selectable_value(&mut self.memory_cfg_gui, MemoryConfig::Ram576, "576 KB");
                                ui.selectable_value(&mut self.memory_cfg_gui, MemoryConfig::Ram608, "608 KB");
                                ui.selectable_value(&mut self.memory_cfg_gui, MemoryConfig::Ram640, "640 KB");
                            });

                        let (new_sw1, new_sw2) = self.memory_cfg_gui.to_switches(&self.sw1, &self.sw2);
                        self.sw1 = u8_to_bool_array(new_sw1);
                        self.sw2 = u8_to_bool_array(new_sw2);

                        ui.end_row();

                        // TODO
                        ui.label("Drive number");
                        egui::ComboBox::from_id_salt("select_dd_num")
                            .selected_text(format!("{:?}", self.dd_num_cfg_gui))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.dd_num_cfg_gui, DDNumConfig::Zero, "0");
                                ui.selectable_value(&mut self.dd_num_cfg_gui, DDNumConfig::One, "1");
                                ui.selectable_value(&mut self.dd_num_cfg_gui, DDNumConfig::Two, "2");
                                ui.selectable_value(&mut self.dd_num_cfg_gui, DDNumConfig::Three, "3");
                                ui.selectable_value(&mut self.dd_num_cfg_gui, DDNumConfig::Four, "4");
                            });

                        self.dd_num_cfg_gui.to_switches(&mut self.sw1);
                            
                        ui.end_row();

                        ui.label("Video card type");
                        egui::ComboBox::from_id_salt("select_video_card_type")
                            .selected_text(format!("{:?}", self.video_card_type))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.video_card_type, VideoCardTypeConfig::Mda8025, "MDA 80x25");
                                ui.selectable_value(&mut self.video_card_type, VideoCardTypeConfig::Cga4025, "CGA 40x25");
                                ui.selectable_value(&mut self.video_card_type, VideoCardTypeConfig::Cga8025, "CGA 80x25");
                            });

                        self.video_card_type.to_switches(&mut self.sw1);
                            
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

                });

                ui.menu_button("EGA", |ui| {
                    if ui.button("320x200_16").clicked() {
                        self.show_ega_320x250_16 = !self.show_ega_320x250_16;
                    }

                    if ui.button("640x350_16").clicked() {
                        self.show_ega_640x350_16 = !self.show_ega_640x350_16;
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

#[derive(PartialEq)]
enum MemoryConfig {
    Ram16,
    Ram32,
    Ram48,
    Ram64,
    Ram96,
    Ram128,
    Ram160,
    Ram192,
    Ram224,
    Ram256,
    Ram288,
    Ram320,
    Ram352,
    Ram384,
    Ram416,
    Ram448,
    Ram480,
    Ram512,
    Ram544,
    Ram576,
    Ram608,
    Ram640,
}

impl MemoryConfig {
    fn from_switches(sw1: u8, sw2: u8) -> Self {
        let bancks_populated = (sw1 & MEM_64K) >> 2;
        let total_ram = sw2 & 0b00011111;

        match bancks_populated {
            0b00 => Self::Ram16,
            0b01 => Self::Ram32,
            0b10 => Self::Ram48,
            0b11 => match total_ram {
                TOTAL_RAM_64 => Self::Ram64,
                TOTAL_RAM_96 => Self::Ram96,
                TOTAL_RAM_128 => Self::Ram128,
                TOTAL_RAM_160 => Self::Ram160,
                TOTAL_RAM_192 => Self::Ram192,
                TOTAL_RAM_224 => Self::Ram224,
                TOTAL_RAM_256 => Self::Ram256,
                TOTAL_RAM_288 => Self::Ram288,
                TOTAL_RAM_320 => Self::Ram320,
                TOTAL_RAM_352 => Self::Ram352,
                TOTAL_RAM_384 => Self::Ram384,
                TOTAL_RAM_416 => Self::Ram416,
                TOTAL_RAM_448 => Self::Ram448,
                TOTAL_RAM_480 => Self::Ram480,
                TOTAL_RAM_512 => Self::Ram512,
                TOTAL_RAM_544 => Self::Ram544,
                TOTAL_RAM_576 => Self::Ram576,
                TOTAL_RAM_608 => Self::Ram608,
                TOTAL_RAM_640 => Self::Ram640,

                _ => panic!("INVALID RAM INITIAL CONFIG")
            }
            _ => unreachable!()
        }

    }

    fn to_switches(&self, sw1: &[bool; 8], sw2: &[bool; 8]) -> (u8, u8) {
        let mut sw1 = bool_array_to_u8(sw1);
        let mut sw2 = bool_array_to_u8(sw2);

        let (new_sw1, new_sw2) = match self {
            MemoryConfig::Ram16 => (0b00, 0b00000),
            MemoryConfig::Ram32 => (0b01, 0b00000),
            MemoryConfig::Ram48 => (0b10, 0b00000),
            MemoryConfig::Ram64 => (0b11, 0b00000),
            MemoryConfig::Ram96 => (0b11, TOTAL_RAM_96),
            MemoryConfig::Ram128 => (0b11, TOTAL_RAM_128),
            MemoryConfig::Ram160 => (0b11, TOTAL_RAM_160),
            MemoryConfig::Ram192 => (0b11, TOTAL_RAM_192),
            MemoryConfig::Ram224 => (0b11, TOTAL_RAM_224),
            MemoryConfig::Ram256 => (0b11, TOTAL_RAM_256),
            MemoryConfig::Ram288 => (0b11, TOTAL_RAM_288),
            MemoryConfig::Ram320 => (0b11, TOTAL_RAM_320),
            MemoryConfig::Ram352 => (0b11, TOTAL_RAM_352),
            MemoryConfig::Ram384 => (0b11, TOTAL_RAM_384),
            MemoryConfig::Ram416 => (0b11, TOTAL_RAM_416),
            MemoryConfig::Ram448 => (0b11, TOTAL_RAM_448),
            MemoryConfig::Ram480 => (0b11, TOTAL_RAM_480),
            MemoryConfig::Ram512 => (0b11, TOTAL_RAM_512),
            MemoryConfig::Ram544 => (0b11, TOTAL_RAM_544),
            MemoryConfig::Ram576 => (0b11, TOTAL_RAM_576),
            MemoryConfig::Ram608 => (0b11, TOTAL_RAM_608),
            MemoryConfig::Ram640 => (0b11, TOTAL_RAM_640),
        };

        sw1 = (sw1 & 0b11110011) | (new_sw1 << 2);
        sw2 = (sw2 & 0b11100000) | new_sw2;

        (sw1, sw2)
    }

}

impl Debug for MemoryConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            MemoryConfig::Ram16 => "16 KB",
            MemoryConfig::Ram32 => "32 KB",
            MemoryConfig::Ram48 => "48 KB",
            MemoryConfig::Ram64 => "64 KB",
            MemoryConfig::Ram96 => "96 KB",
            MemoryConfig::Ram128 => "128 KB",
            MemoryConfig::Ram160 => "160 KB",
            MemoryConfig::Ram192 => "192 KB",
            MemoryConfig::Ram224 => "224 KB",
            MemoryConfig::Ram256 => "256 KB",
            MemoryConfig::Ram288 => "288 KB",
            MemoryConfig::Ram320 => "320 KB",
            MemoryConfig::Ram352 => "352 KB",
            MemoryConfig::Ram384 => "384 KB",
            MemoryConfig::Ram416 => "416 KB",
            MemoryConfig::Ram448 => "448 KB",
            MemoryConfig::Ram480 => "480 KB",
            MemoryConfig::Ram512 => "512 KB",
            MemoryConfig::Ram544 => "544 KB",
            MemoryConfig::Ram576 => "576 KB",
            MemoryConfig::Ram608 => "608 KB",
            MemoryConfig::Ram640 => "640 KB",
        };

        write!(f, "{}", val)
    }
}

#[derive(PartialEq)]
#[repr(u8)]
enum DDNumConfig {
    Zero,
    One,
    Two,
    Three,
    Four
}

impl DDNumConfig {
    fn from_switches(sw1: u8) -> Self {
        let sw1_1 = sw1 & 1;
        let sw1_78 = (sw1 & 0b11000000) >> 6;

        if sw1_1 == 0 {
            Self::Zero
        } else {
            match sw1_78 {
                0b00 => Self::One,
                0b01 => Self::Two,
                0b10 => Self::Three,
                0b11 => Self::Four,

                _ => unreachable!()
            }
        }
    }

    fn to_switches(&self, sw1: &mut [bool; 8]) {
        match self {
            Self::Zero => {
                sw1[0] = false;
            }
            Self::One => {
                sw1[0] = true;
                sw1[6] = true;
                sw1[7] = true;
            }
            Self::Two => {
                sw1[0] = true;
                sw1[6] = false;
                sw1[7] = true;
            }
            Self::Three => {
                sw1[0] = true;
                sw1[6] = true;
                sw1[7] = false;
            }
            Self::Four => {
                sw1[0] = true;
                sw1[6] = false;
                sw1[7] = false;
            }
        }
    }
}

impl Debug for DDNumConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Self::Zero => 0,
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4
        };

        write!(f, "{}", val)
    }
}

#[derive(PartialEq)]
enum VideoCardTypeConfig {
    Mda8025,
    Cga4025,
    Cga8025,
    Reserved
}

impl VideoCardTypeConfig {
    fn from_switches(sw1: u8) -> Self {
        let card_type = (sw1 & 0b00110000) >> 4;

        match card_type {
            0b00 => Self::Reserved,
            0b01 => Self::Cga8025,
            0b10 => Self::Cga4025,
            0b11 => Self::Mda8025,

            _ => unreachable!()
        }
    }

    fn to_switches(&self, sw1: &mut [bool; 8]) {
        match self {
            Self::Cga4025 => {
                sw1[4] = true;
                sw1[5] = false;
            },
            Self::Cga8025 => {
                sw1[4] = false;
                sw1[5] = true;
            },
            Self::Mda8025 => {
                sw1[4] = true;
                sw1[5] = true;
            },
            Self::Reserved => {
                sw1[4] = false;
                sw1[5] = false;
            }
        };
    }
}

impl Debug for VideoCardTypeConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Self::Cga4025 => "CGA 40x25",
            Self::Cga8025 => "CGA 80x25",
            Self::Mda8025 => "MDA 80x25",
            Self::Reserved => "Reserved"
        };

        write!(f, "{}", val)
    }
}

pub struct EmulatorConfig {
    sw1: u8,
    sw2: u8,
    screen_mode: ScreenMode,
}

impl Default for EmulatorConfig {
    fn default() -> Self {
        let sw1 = DD_ENABLE | RESERVED | MEM_64K | DISPLAY_MDA_80_25 | DRIVES_2;
        let sw2 = HIGH_NIBBLE | TOTAL_RAM_640;
        Self {
            sw1, sw2,
            screen_mode: ScreenMode::default(),
        }
    }
}

impl EmulatorConfig {
    pub fn builder() -> EmulatorConfig {
        Self {
            sw1: 0,
            sw2: HIGH_NIBBLE,
            screen_mode: ScreenMode::default(),
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
        let sw1 = match screen_mode.variant {
            ScreenModeVariant::MDA8025 => Some(DISPLAY_MDA_80_25),
            ScreenModeVariant::CGA4025 => Some(DISPLAY_CGA_40_25),
            ScreenModeVariant::CGA8025 => Some(DISPLAY_CGA_80_25),
            ScreenModeVariant::EGA320X200X16 => Some(DISPLAY_EGA),
            ScreenModeVariant::EGA640X350X16 => Some(DISPLAY_EGA),

            // _ => None,
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
