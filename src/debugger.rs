use crate::{hardware::sys::System, util::disassembly::Disassembler};
use egui::{Key, Ui};
use egui_extras::{Size, StripBuilder};
use lazy_static::lazy_static;
use regex::{Regex, RegexSet};

lazy_static! {
    static ref PATTERNS: Vec<&'static str> = vec![
        r"([[:xdigit:]]{1,4}):([[:xdigit:]]{1,4})",
        r"([[:xdigit:]]{1,5})",
    ];
    static ref SET: RegexSet = RegexSet::new(PATTERNS.iter()).unwrap();
    static ref REGEXES: Vec<Regex> = SET
        .patterns()
        .iter()
        .map(|pat| Regex::new(pat).unwrap())
        .collect();
}

fn addr_match(text: &str) -> Vec<usize> {
    SET.matches(text).into_iter().collect()
}

pub struct MyApp {
    sys: System,

    running_texts: [String; 2],

    show_add_str: String,
    show_add_init: usize,

    disassembler: Disassembler,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut app = Self {
            sys: System::default(),

            running_texts: [String::from("Run"), String::from("Stop")],

            show_add_str: String::new(),
            show_add_init: 0xFE000,

            disassembler: Disassembler::default(),
        };

        app.sys
            .disk_ctrl
            .insert_disk(&mut app.sys.bus, 0, "roms/dos/Disk01.img");
        app.sys
            .disk_ctrl
            .insert_disk(&mut app.sys.bus, 1, "roms/dos/Disk02.img");

        app
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("Actions", |ui| {
                    self.action_buttons(ui);
                });
            });

            ui.separator();

            StripBuilder::new(ui)
                .size(Size::exact(222.)) // top cell
                .size(Size::remainder()) // bottom cell
                .vertical(|mut strip| {
                    // Add the top 'cell'
                    strip.strip(|builder| {
                        builder
                            .size(Size::relative(0.322))
                            .size(Size::relative(0.12))
                            .size(Size::relative(0.558))
                            .horizontal(|mut strip| {
                                strip.cell(|ui| {
                                    self.disassembly_view(ui);
                                });
                                strip.cell(|ui| {
                                    self.control_buttons(ui);
                                });
                                strip.cell(|ui| {
                                    self.memory_view(ui);
                                });
                            });
                    });
                    // We add a nested strip in the bottom cell:
                    strip.strip(|builder| {
                        builder.sizes(Size::remainder(), 2).horizontal(|mut strip| {
                            strip.cell(|ui| {
                                self.reg_view(ui);
                            });
                            strip.cell(|ui| {
                                ui.label("Top Right");
                            });
                        });
                    });
                });
        });
    }
}

impl MyApp {
    fn action_buttons(&mut self, ui: &mut Ui) {
        if ui.button("Load ROMs").clicked() {
            self.sys.load_roms();
            // self.disassembler.run(8, &self.sys.bus.memory, &self.sys.cpu);
            self.disassembler.clear_cache();
            ui.close_menu();
        }

        if ui.button("Load test").clicked() {
            self.sys.load_test("roms/tests/jmpmov.bin");

            self.disassembler.clear_cache();
            ui.close_menu();
        }

        if ui.button("Reset").clicked() {
            self.sys.rst();
            // self.disassembler.run(8, &self.sys.bus.memory, &self.sys.cpu);
            self.disassembler.clear_cache();
            ui.close_menu();
        }
    }

    fn control_buttons(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.ctx().request_repaint();

            ui.vertical_centered_justified(|ui| {
                ui.add_enabled_ui(!self.sys.running, |ui| {
                    if ui.button("Step").clicked() {
                        self.sys.step(&mut 0);
                    }
                });

                if ui
                    .button(&self.running_texts[self.sys.running as usize])
                    .clicked()
                {
                    self.disassembler
                        .run(8, &self.sys.bus.memory, &self.sys.cpu);
                    self.sys.running = !self.sys.running;
                }
            });

            if self.sys.running {
                self.sys.update_debugger();
            }
        });
    }

    fn memory_view(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.vertical_centered_justified(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("Memory");
                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut self.show_add_str)
                                .hint_text("Write something here"),
                        )
                        .lost_focus()
                        && ui.input(|i| i.key_pressed(Key::Enter))
                    {
                        self.parse_address();
                    };
                });

                egui::Grid::new("mem_grid").striped(true).show(ui, |ui| {
                    ui.label("");
                    let index_str: String = (0u8..16).map(|val| format!("{:02X} ", val)).collect();
                    ui.label(egui::RichText::new(index_str).text_style(egui::TextStyle::Monospace));
                    ui.end_row();

                    for i in 0..8 {
                        let addr = (self.show_add_init + i * 0x10) % 0x100000;

                        ui.label(
                            egui::RichText::new(&format!("{:05X} | ", addr))
                                .text_style(egui::TextStyle::Monospace),
                        );

                        let mut mem_view = String::with_capacity(100);

                        for j in 0..0x10 {
                            mem_view.push_str(&format!("{:02X} ", self.sys.bus.memory[addr + j]));
                        }

                        let ascii: String = self.sys.bus.memory[addr..addr + 0x10]
                            .iter()
                            .map(|val| special_char(val))
                            .collect();

                        ui.label(
                            egui::RichText::new(&mem_view).text_style(egui::TextStyle::Monospace),
                        );
                        ui.label(
                            egui::RichText::new(&ascii).text_style(egui::TextStyle::Monospace),
                        );
                        ui.end_row();
                    }
                });
            });
        });
    }

    fn disassembly_view(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.vertical_centered_justified(|ui| {
                ui.heading("Disassembly");
                ui.label("");
                let instrs = self
                    .disassembler
                    .run(8, &self.sys.bus.memory, &self.sys.cpu);

                egui::Grid::new("disassembly_grid").show(ui, |ui| {
                    for i in instrs {
                        ui.label(
                            egui::RichText::new(String::from(format!("{:05X} | ", i.2)))
                                .text_style(egui::TextStyle::Monospace),
                        );
                        ui.label(
                            egui::RichText::new(String::from(format!("{:010} | ", i.1)))
                                .text_style(egui::TextStyle::Monospace),
                        );
                        ui.label(
                            egui::RichText::new(String::from(format!("{:020}", i.0)))
                                .text_style(egui::TextStyle::Monospace),
                        );
                        ui.end_row()
                    }
                });

                // ui.label(egui::RichText::new(&format!("{:020}", format!("{}", self.sys.cpu.instr))).text_style(egui::TextStyle::Monospace));
            });
        });
    }

    fn reg_view(&self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label("General purpose registers");
                egui::Grid::new("gpregs").show(ui, |ui| {
                    ui.label(egui::RichText::new("AX").text_style(egui::TextStyle::Monospace));
                    ui.label(
                        egui::RichText::new(format!("{:04X}", self.sys.cpu.ax.get_x()))
                            .text_style(egui::TextStyle::Monospace),
                    );
                    ui.end_row();
                    ui.label(egui::RichText::new("BX").text_style(egui::TextStyle::Monospace));
                    ui.label(
                        egui::RichText::new(format!("{:04X}", self.sys.cpu.bx.get_x()))
                            .text_style(egui::TextStyle::Monospace),
                    );
                    ui.end_row();
                    ui.label(egui::RichText::new("CX").text_style(egui::TextStyle::Monospace));
                    ui.label(
                        egui::RichText::new(format!("{:04X}", self.sys.cpu.cx.get_x()))
                            .text_style(egui::TextStyle::Monospace),
                    );
                    ui.end_row();
                    ui.label(egui::RichText::new("DX").text_style(egui::TextStyle::Monospace));
                    ui.label(
                        egui::RichText::new(format!("{:04X}", self.sys.cpu.dx.get_x()))
                            .text_style(egui::TextStyle::Monospace),
                    );
                    ui.end_row();
                });

                ui.label("Index registers");
                egui::Grid::new("iregs").show(ui, |ui| {
                    ui.label(egui::RichText::new("SI").text_style(egui::TextStyle::Monospace));
                    ui.label(
                        egui::RichText::new(format!("{:04X}", self.sys.cpu.si))
                            .text_style(egui::TextStyle::Monospace),
                    );
                    ui.end_row();
                    ui.label(egui::RichText::new("DI").text_style(egui::TextStyle::Monospace));
                    ui.label(
                        egui::RichText::new(format!("{:04X}", self.sys.cpu.di))
                            .text_style(egui::TextStyle::Monospace),
                    );
                    ui.end_row();
                    ui.label(egui::RichText::new("BP").text_style(egui::TextStyle::Monospace));
                    ui.label(
                        egui::RichText::new(format!("{:04X}", self.sys.cpu.bp))
                            .text_style(egui::TextStyle::Monospace),
                    );
                    ui.end_row();
                    ui.label(egui::RichText::new("SP").text_style(egui::TextStyle::Monospace));
                    ui.label(
                        egui::RichText::new(format!("{:04X}", self.sys.cpu.sp))
                            .text_style(egui::TextStyle::Monospace),
                    );
                    ui.end_row();
                });

                ui.label("Flags");
                ui.label(
                    egui::RichText::new(format!("{:016b}", self.sys.cpu.flags.get_flags()))
                        .text_style(egui::TextStyle::Monospace),
                );
                ui.label(
                    egui::RichText::new("----ODITSZ-A-P-C").text_style(egui::TextStyle::Monospace),
                );

                ui.label("Segment registers");
                egui::Grid::new("sregs").show(ui, |ui| {
                    ui.label(egui::RichText::new("CS").text_style(egui::TextStyle::Monospace));
                    ui.label(
                        egui::RichText::new(format!("{:04X}", self.sys.cpu.cs))
                            .text_style(egui::TextStyle::Monospace),
                    );
                    ui.end_row();
                    ui.label(egui::RichText::new("DS").text_style(egui::TextStyle::Monospace));
                    ui.label(
                        egui::RichText::new(format!("{:04X}", self.sys.cpu.ds))
                            .text_style(egui::TextStyle::Monospace),
                    );
                    ui.end_row();
                    ui.label(egui::RichText::new("ES").text_style(egui::TextStyle::Monospace));
                    ui.label(
                        egui::RichText::new(format!("{:04X}", self.sys.cpu.es))
                            .text_style(egui::TextStyle::Monospace),
                    );
                    ui.end_row();
                    ui.label(egui::RichText::new("SS").text_style(egui::TextStyle::Monospace));
                    ui.label(
                        egui::RichText::new(format!("{:04X}", self.sys.cpu.ss))
                            .text_style(egui::TextStyle::Monospace),
                    );
                    ui.end_row();
                });

                ui.label("Instruction Pointer");
                ui.label(
                    egui::RichText::new(format!("{:04X}", self.sys.cpu.ip))
                        .text_style(egui::TextStyle::Monospace),
                );
            });
        });
    }

    fn parse_address(&mut self) {
        let matches = addr_match(&self.show_add_str);

        if matches.is_empty() {
            log::info!("ERROR SINTAXIS DIRECCION");
            self.show_add_str.clear();
            return;
        }

        let first = matches.first().unwrap();

        for cap in REGEXES[*first].captures_iter(&self.show_add_str) {
            match first {
                0 => {
                    let high = usize::from_str_radix(&cap[1], 16).unwrap();
                    let low = usize::from_str_radix(&cap[2], 16).unwrap();
                    self.show_add_init = ((high << 4) + low) & 0xFFFF0;
                }
                1 => {
                    self.show_add_init = usize::from_str_radix(&cap[1], 16).unwrap() & 0xFFFF0;
                }
                _ => unreachable!(),
            };
        }

        log::info!("{:?}", matches);
    }
}

fn special_char(chr: &u8) -> char {
    if (32..127).contains(chr) {
        *chr as char
    } else {
        '.'
    }
}
