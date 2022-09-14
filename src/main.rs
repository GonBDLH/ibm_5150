mod hardware;
mod util;

// use std::{sync::{Arc, mpsc::{Sender, Receiver, self}, RwLock}, thread::JoinHandle};

// use std::time::Duration;

use ggez::conf::WindowMode;
// use eframe::{run_native, NativeOptions, App};
use hardware::sys::System;
// use util::debug::display;

// struct IbmPc {
//     system: Arc<RwLock<System>>,

//     running: bool,
//     run_handle: Option<JoinHandle<()>>,
//     tx: Sender<bool>,
// }

// impl IbmPc {
//     pub fn new(system: Arc<RwLock<System>>, tx: Sender<bool>) -> Self {
//         IbmPc { 
//             system,

//             running: false,
//             run_handle: None,
//             tx,
//         }
//     }
// }

// impl App for IbmPc {
//     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
//         egui::CentralPanel::default().show(ctx, |ui| {
//             if let Some(v) = &self.run_handle {
//                 self.running = !v.is_finished();
//             }

//             if ui.button("Step").clicked() {
//                 if !self.running {
//                     self.system.write().unwrap().step();
//                 }
//             }

//             let ip = match self.system.try_read() {
//                 Ok(v) => v.cpu.ip,
//                 Err(_) => 0,
//             };

//             let halted = match self.system.try_read() {
//                 Ok(v) => v.cpu.halted,
//                 Err(_) => false,
//             };

//             ui.label(format!("{}", ip));
//             ui.label(format!("{}", halted));

//             if ui.button("Run").clicked() {
//                 let sys_thread = self.system.clone();

//                 self.tx.send(!self.running).unwrap();
//                 self.running = !self.running;

//                 if self.running {
//                     self.run_handle = Some(std::thread::spawn(move || {
//                         sys_thread.write().unwrap().run();
//                     }));
//                 }
//             }

//             ui.label(format!("{}", self.running));

//             if ui.button("Reset").clicked() {
//                 if !self.running {
//                     self.system.write().unwrap().rst();
//                     self.system.write().unwrap().load_bios();
//                 }
//             }
//         });

//         // display(&self.system.read().unwrap());
//     }
// }

// fn main() {
//     let (tx, rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();

//     let sys = Arc::new(RwLock::new(System::new(rx)));

//     sys.write().unwrap().load_bios();

//     let app = IbmPc::new(sys.clone(), tx);

//     run_native("IBM PC", NativeOptions::default(), Box::new(|_cc| Box::new(app)));
// }

const DESIRED_FPS: f32 = 50.;

use ggez::{GameError, GameResult};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color};
use ggez::timer::check_update_time;

struct IbmPc {
    sys: System,
}

impl IbmPc {
    pub fn new() -> Self {
        IbmPc {
            sys: System::new()
        }
    }
}

impl EventHandler for IbmPc {
    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        let mut veces = 0;

        while check_update_time(ctx, DESIRED_FPS as u32) {
            self.sys.update();
            veces += 1;
        }

        println!("{veces} - {}", ggez::timer::fps(ctx));

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        graphics::clear(ctx, Color::WHITE);
        // TODO

        graphics::present(ctx)
    }
}

fn _main() -> GameResult {
    let mut app = IbmPc::new();
    let cb = ggez::ContextBuilder::new("IBM 5150", "Gonzalo");
 
    let win_mode = WindowMode {
        width: 720.,
        height: 350.,
        resizable: false,
        ..Default::default()
    };
 
    let (mut ctx, event_loop) = cb.build()?;

    graphics::set_mode(&mut ctx, win_mode)?;

    app.sys.rst();
    app.sys.load_bios();

    event::run(ctx, event_loop, app);
}

fn main() {
    let mut app = IbmPc::new();

    app.sys.rst();
    app.sys.load_bios();

    loop {
        app.sys.update();
    }
}
