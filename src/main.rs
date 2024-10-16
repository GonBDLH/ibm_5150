use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use eframe::glow::Texture;
use eframe::CreationContext;
use egui::{Color32, ColorImage, Context, ImageData, Label, TextureHandle, TextureOptions, Ui};
use ibm_5150::frontend::*;
use ibm_5150::hardware::switches_cfg::*;
// use ibm_5150::{/*debugger::*,*/ hardware::sys::System};

use ibm_5150::hardware::sys::{ScreenMode, System};
use winit::event_loop::EventLoop;

// use egui::{vec2, ViewportBuilder};

// fn main_debugger() -> Result<(), eframe::Error> {
//     let options = eframe::NativeOptions {
//         viewport: ViewportBuilder::default().with_resizable(false).with_inner_size(vec2(1000., 700.)),
//         ..Default::default()
//     };

//     eframe::run_native("Prueba", options, Box::new(|_cc| Box::<MyApp>::default()))
// }

// fn main_view() -> GameResult {
//     let sw1 = DD_ENABLE | RESERVED | MEM_64K | DISPLAY_CGA_40_25 | DRIVES_2;
//     let sw2 = HIGH_NIBBLE | PLUS_32;

//     let dimensions = match sw1 & 0b00110000 {
//         DISPLAY_RESERVED => panic!("Reserved"),
//         DISPLAY_CGA_40_25 => (320., 200.),
//         DISPLAY_CGA_80_25 => (640., 200.),
//         DISPLAY_MDA_80_25 => (720., 350.),
//         _ => unreachable!(),
//     };

//     let win_mode = WindowMode::default()
//         .dimensions(dimensions.0 * 2., dimensions.1 * 2.)
//         .resize_on_scale_factor_change(true);

//     let win_setup = WindowSetup::default().srgb(true);

//     let cb = ggez::ContextBuilder::new("IBM 5150", "Gonzalo")
//         .window_mode(win_mode)
//         .window_setup(win_setup);

//     let (ctx, event_loop) = cb.build()?;

//     let mut app = IbmPc::new(&ctx, sw1, sw2, dimensions);
//     //graphics::set_mode(&mut ctx, win_mode)?;

//     app.sys.rst();
//     app.sys.load_roms();

//     app.sys
//         .disk_ctrl
//         .insert_disk(&mut app.sys.bus, 0, "roms/dos/3.00/Disk01.img");
//     app.sys
//         .disk_ctrl
//         .insert_disk(&mut app.sys.bus, 1, "roms/dos/3.00/Disk02.img");
//     // app.sys
//     //     .disk_ctrl
//     //     .insert_disk(&mut app.sys.bus, 1, "roms/personal_editor/Disk1.img");
//     // app.sys
//     //     .disk_ctrl
//     //     .insert_disk(&mut app.sys.bus, 0, "roms/dos/1.10/DISK01.IMA");

//     event::run(ctx, event_loop, app);
// }

// fn main() {
//     let args: Vec<String> = env::args().collect();
//     env_logger::init();

//     match args.len() {
//         1 => main_view().unwrap(),
//         2 => match args[1].as_str() {
//             "debugger" => main_debugger().unwrap(),
//             _ => panic!("Wrong arguments"),
//         },
//         _ => panic!("Wrong arguments"),
//     }
// }

// fn main() -> Result<(), impl std::error::Error> {
//     let event_loop = EventLoop::new().unwrap();

//     let sw1 = DD_ENABLE | RESERVED | MEM_64K | DISPLAY_CGA_40_25 | DRIVES_2;
//     let sw2 = HIGH_NIBBLE | TOTAL_RAM_640;

//     let screen_mode = match sw1 & 0b00110000 {
//         DISPLAY_RESERVED => panic!("Reserved"),
//         DISPLAY_CGA_40_25 => ScreenMode::CGA4025,
//         DISPLAY_CGA_80_25 => ScreenMode::CGA8025,
//         DISPLAY_MDA_80_25 => ScreenMode::MDA4025,
//         _ => unreachable!(),
//     };

//     simple_logger::SimpleLogger::new().env().init().unwrap();

//     let mut app = IbmPc::new(sw1, sw2, screen_mode);

//     app.sys.rst();
//     app.sys.load_roms();

//     // app.sys
//     //     .disk_ctrl
//     //     .insert_disk(&mut app.sys.bus, 0, "roms/dos/2.10/Disk01.img");
//     // app.sys
//     //     .disk_ctrl
//     //     .insert_disk(&mut app.sys.bus, 1, "roms/dos/2.10/Disk02.img");

//     // app.sys
//     //     .disk_ctrl
//     //     .insert_disk(&mut app.sys.bus, 0, "roms/dos/FreeDOS/FreeDOS 1.3 Disk 1.img");
//     // app.sys
//     //     .disk_ctrl
//     //     .insert_disk(&mut app.sys.bus, 1, "roms/dos/FreeDOS/FreeDOS 1.3 Disk 2.img");

//     app.sys
//         .disk_ctrl
//         .insert_disk(&mut app.sys.bus, 0, "roms/otros/FS.img");

//     // app.sys
//     //     .disk_ctrl
//     //     .insert_disk(&mut app.sys.bus, 0, "roms/otros/101-MONOCHROME-MAZES.img");

//     event_loop.run_app(&mut app)
// }

fn main() -> eframe::Result {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000., 1000.]),
        ..Default::default()
    };

    let sw1 = DD_ENABLE | RESERVED | MEM_64K | DISPLAY_MDA_80_25 | DRIVES_2;
    let sw2 = HIGH_NIBBLE | TOTAL_RAM_640;

    let screen_mode = match sw1 & 0b00110000 {
        DISPLAY_RESERVED => panic!("Reserved"),
        DISPLAY_CGA_40_25 => ScreenMode::CGA4025,
        DISPLAY_CGA_80_25 => ScreenMode::CGA8025,
        DISPLAY_MDA_80_25 => ScreenMode::MDA4025,
        _ => unreachable!(),
    };

    let mut app = IbmPc::new(sw1, sw2, screen_mode);

    app.run_emulation();

    eframe::run_native(
        "IBM 5150 Emulator",
        options,
        Box::new(|cc| {
            // egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::new(app.set_texture_handles(cc)))
        }),
    )
}

#[derive(Default)]
struct IbmPc {
    sys: Arc<Mutex<System>>,

    show_mda: bool,
    show_cga_40x25: bool,
    show_cga_80x25: bool,

    floppy_path: [Option<PathBuf>; 2],

    texture_handles: Option<[TextureHandle; 3]>,
}

impl eframe::App for IbmPc {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_pannel").show(ctx, |ui| {
            self.add_menu(ctx, ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.add_screen_windows(ctx);
        });
    }
}

impl IbmPc {
    pub fn new(sw1: u8, sw2: u8, screen_mode: ScreenMode) -> Self {
        Self {
            sys: Arc::new(Mutex::new(System::new(sw1, sw2, screen_mode))),
            ..Default::default()
        }
    }

    fn run_emulation(&mut self) {
        let sys = self.sys.clone();

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

    pub fn set_texture_handles(mut self, cc: &CreationContext) -> Self {
        let texture_handles = [
            cc.egui_ctx.load_texture(
                "mda_40x25",
                ImageData::Color(Arc::new(ColorImage::new([720, 350], Color32::BLACK))),
                TextureOptions::NEAREST,
            ),
            cc.egui_ctx.load_texture(
                "cga_40x25",
                ImageData::Color(Arc::new(ColorImage::new([320, 200], Color32::BLACK))),
                TextureOptions::NEAREST,
            ),
            cc.egui_ctx.load_texture(
                "cga_80x25",
                ImageData::Color(Arc::new(ColorImage::new([640, 200], Color32::BLACK))),
                TextureOptions::NEAREST,
            ),
        ];

        self.texture_handles = Some(texture_handles);

        self
    }

    fn add_menu(&mut self, ctx: &Context, ui: &mut Ui) {
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

            ui.menu_button("Debug", |ui| {});

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

    fn add_screen_windows(&mut self, ctx: &Context) {
        let size = (720. / 1.125, 350. * 1.33 / 1.125);

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

    fn load_floppy_button(&mut self, ui: &mut Ui, floppy: usize) {
        if self.floppy_path[floppy].is_none() {
            if ui.button("Open file...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.floppy_path[floppy] = Some(path.clone());

                    // println!("{:?}", path.to_str());

                    // self.sys.lock().unwrap().disk_ctrl.insert_disk(
                    //     &mut self.sys.lock().unwrap().bus,
                    //     floppy,
                    //     path.to_str().unwrap(),
                    // );
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
