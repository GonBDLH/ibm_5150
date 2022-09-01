// use std::fs::File;
use std::io::stdout;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;
#[cfg(not(debug_assertions))]
use std::time::{Instant, Duration};
// use std::thread::sleep;
// use std::time::{Instant, Duration};

use crossterm::execute;
use crossterm::terminal::SetSize;

use super::cpu_8088::CPU;
use super::bus::Bus;
use crate::util::debug::*;

const FPS: f32 = 50.0;

pub struct System {
    pub cpu: CPU,
    pub bus: Bus,

    pub running: bool,

    pub rx: Mutex<Receiver<bool>>,
}

impl System {
    pub fn new(rx: Receiver<bool>) -> Self {
        execute!(stdout(), SetSize(120, 30)).unwrap();
        let sys = System { 
            cpu: CPU::new(),
            bus: Bus::new(),

            running: false,

            rx: Mutex::new(rx),
        };
        
        sys
    }
}

impl System {
    pub fn rst(&mut self) {
        self.cpu = CPU::new();
        self.bus = Bus::new();

        self.running = false;
    }

    // Llamar 60 veces por segundo
    pub fn update(self: &mut Self) {
        let max_cycles = (4_772_726.7 / FPS) as u32;
        let mut cycles_ran = 0;

        while cycles_ran <= max_cycles {
            let cycles = self.cpu.fetch_decode_execute(&mut self.bus);
            cycles_ran += cycles;

            // RESTO DE UPDATES (TIMERS, ETC)
            self.bus.update_peripherals(cycles);

            self.cpu.handle_interrupts(&mut self.bus);

            if self.cpu.halted { 
                let _a = 0;
                break; 
            }
        }

        display(self);
    }

    pub fn step(self: &mut Self) {
        let cycles = self.cpu.fetch_decode_execute(&mut self.bus);

        // RESTO DE UPDATES (TIMERS, ETC)
        self.bus.update_peripherals(cycles);

        self.cpu.handle_interrupts(&mut self.bus);

        display(self);
    }

    pub fn run(&mut self) {
        self.running = self.rx.lock().unwrap().recv().unwrap();

        while self.running {
            #[cfg(not(debug_assertions))]
            let start = Instant::now();

            self.update();

            self.running = match self.rx.lock().unwrap().try_recv() {
                Ok(v) => v,
                Err(_v) => self.running,
            };

            if self.cpu.halted {
                self.running = false;
            };

            #[cfg(not(debug_assertions))] {
                let end = Instant::now();

                let t = end.duration_since(start).as_millis();
                let millis = ((1. / FPS) * 1000.) as u128;
                std::thread::sleep(Duration::from_millis((millis - t) as u64));
            }
        }
    }

    pub fn load_bios(&mut self) {
        unsafe {
            for (idx, element) in std::fs::read("roms/bios.bin").unwrap().into_iter().enumerate() {
                std::ptr::write(&mut self.bus.memory[0xF6000 + idx], element);
            }
    
            for (idx, element) in std::fs::read("roms/bios.BIN").unwrap().into_iter().enumerate() {
                std::ptr::write(&mut self.bus.memory[0xFE000 + idx], element);
            }
        }
    }
}
