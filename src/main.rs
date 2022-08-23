mod hardware;
mod util;

use std::sync::{Arc, Mutex};

use eframe::{run_native, NativeOptions, App};
use hardware::sys::System;

struct GUI;

impl App for GUI {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        
    }
}

fn main() {
    let native_options = NativeOptions::default();
    let app = GUI;

    // let mut sys = Arc::new(Mutex::new(System::new()));

    let mut sys = System::new();

    run_native("IBM PC", native_options, Box::new(|_cc| Box::new(GUI)));
    // sys.run();
}
