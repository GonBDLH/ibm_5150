use ibm_5150::debugger::*;
use ibm_5150::screen::*;
use notan::draw::DrawConfig;
use notan::prelude::*;
use std::env;

use egui::vec2;

fn main_debugger() -> Result<(), eframe::Error> {
    let mut options = eframe::NativeOptions::default();

    options.resizable = false;
    options.initial_window_size = Some(vec2(1000., 700.));

    eframe::run_native(
        "Prueba",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}

fn main_view() -> Result<(), String> {
    let win_config = WindowConfig::new().size(720, 350).vsync(true);

    notan::init_with(IbmPc::new)
        .add_config(win_config)
        .add_config(DrawConfig)
        .update(update)
        .draw(draw)
        .event(event)
        .build()
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => main_view().unwrap(),
        2 => match args[1].as_str() {
            "debugger" => main_debugger().unwrap(),
            _ => panic!("Wrong arguments"),
        },
        _ => panic!("Wrong arguments"),
    }
}
