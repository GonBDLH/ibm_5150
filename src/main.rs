mod hardware;
mod util;

use std::{sync::{Arc, mpsc::{Sender, Receiver, self}, RwLock}, thread::JoinHandle};

use eframe::{run_native, NativeOptions, App};
use hardware::sys::System;

struct IbmPc {
    system: Arc<RwLock<System>>,

    running: bool,
    run_handle: Option<JoinHandle<()>>,
    tx: Sender<bool>,
}

impl IbmPc {
    pub fn new(system: Arc<RwLock<System>>, tx: Sender<bool>) -> Self {
        IbmPc { 
            system,

            running: false,
            run_handle: None,
            tx,
        }
    }
}

impl App for IbmPc {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(v) = &self.run_handle {
                self.running = !v.is_finished();
            }

            if ui.button("Step").clicked() {
                if !self.running {
                    self.system.write().unwrap().step();
                }
            }

            ui.label(format!("{}", self.system.read().unwrap().cpu.ip));
            ui.label(format!("{}", self.system.read().unwrap().cpu.halted));

            if ui.button("Run").clicked() {
                let sys_thread = self.system.clone();

                self.tx.send(!self.running).unwrap();
                self.running = !self.running;

                if self.running {
                    self.run_handle = Some(std::thread::spawn(move || {
                        sys_thread.write().unwrap().run();
                    }));
                }
            }

            ui.label(format!("{}", self.running));

            if ui.button("Reset").clicked() {
                if !self.running {
                    self.system.write().unwrap().rst();
                    self.system.write().unwrap().load_bios();
                }
            }
        });
    }
}

fn main() {
    let (tx, rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();

    let sys = Arc::new(RwLock::new(System::new(rx)));

    sys.write().unwrap().load_bios();

    let app = IbmPc::new(sys.clone(), tx);

    run_native("IBM PC", NativeOptions::default(), Box::new(|_cc| Box::new(app)));

    // sys.run();
}
