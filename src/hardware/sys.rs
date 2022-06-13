use std::io::stdout;
use std::thread::sleep;
use std::time::{Instant, Duration};

use crossterm::execute;
use crossterm::terminal::SetSize;

use super::cpu::CPU;
use super::bus::Bus;
use super::debug::*;

pub struct System {
    pub cpu: CPU,
    pub bus: Bus,

    pub running: bool,
}

impl System {
    pub fn new() -> Self {
        execute!(stdout(), SetSize(120, 30)).unwrap();
        System { 
            cpu: CPU::new(),
            bus: Bus::new(),

            running: true,
        }
    }
}

impl System {
    pub fn clock(self: &mut Self) {
        loop {
            let start = Instant::now();
            // for _i in 0..286364 {
            //     if self.clock_cycles % 3 == 0 {
            //         self.cpu.step();
            //     }

            //     if self.clock_cycles % 4 == 0 {
            //         // Aqui CGA
            //     }

            //     if self.clock_cycles % 12 == 0 {
            //         // Aqui timers 8253
            //     }
                
            //     self.clock_cycles += 1;
            // }
            self.cpu.step(&mut self.bus);

            let t = Duration::new(0, 20_000_000).checked_sub(Duration::from(Instant::now() - start)).unwrap_or_default();
            // println!("{}", t.as_micros());
            display(self);
            sleep(t);
        }
    }

    pub fn run(self: &mut Self) {
        while self.running {
            display(self);
            get_command(self);
        }
    }

    pub fn load_bios(&mut self) {
        for (idx, element) in std::fs::read("roms/bios.BIN").unwrap().into_iter().enumerate() {
            unsafe {
                std::ptr::write(&mut self.bus.memory[0xFE000 + idx], element);
            }
        }
        let a = 0;
    }
}