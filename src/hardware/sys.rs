use std::fs::File;
use std::io::{stdout, Write};
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

    pub not_sleeped: u64,

    log: File,
}

impl System {
    pub fn new() -> Self {
        execute!(stdout(), SetSize(120, 30)).unwrap();
        let sys = System { 
            cpu: CPU::new(),
            bus: Bus::new(),

            running: true,

            not_sleeped: 0,

            log: File::create("logs/dirs.txt").unwrap(),
        };
        
        sys
    }
}

impl System {
    // Llamar 60 veces por segundo
    pub fn update(self: &mut Self) {
        let max_cycles = (4_772_726.7 / FPS) as u64;
        let mut cycles_ran = 0;

        while cycles_ran < max_cycles {
            let cycles = self.cpu.fetch_decode_execute(&mut self.bus);
            cycles_ran += cycles;

            // RESTO DE UPDATES (TIMERS, ETC)
            self.bus.pit.tick(cycles);

            self.cpu.handle_interrupts(&mut self.bus);

            // write!(self.log, "{:04X}\n", self.cpu.ip).unwrap();

            if self.cpu.ip == 0xE0F0 {
                let a = 0;
            }

            if self.cpu.halted { break; }
        }

        display(self);
    }

    // pub fn clock(self: &mut Self) {
    //     loop {
    //         let start = Instant::now();
    //         // for _i in 0..286364 {
    //         //     if self.clock_cycles % 3 == 0 {
    //         //         self.cpu.step();
    //         //     }

    //         //     if self.clock_cycles % 4 == 0 {
    //         //         // Aqui CGA
    //         //     }

    //         //     if self.clock_cycles % 12 == 0 {
    //         //         // Aqui timers 8253
    //         //     }
                
    //         //     self.clock_cycles += 1;
    //         // }
    //         self.cpu.step(&mut self.bus);

    //         if self.cpu.halted {
    //             // TODO esto seguramente haya que cambiarlo
    //             return;
    //         }

    //         let t = Duration::new(0, 20_000_000).checked_sub(Duration::from(Instant::now() - start)).unwrap_or_default();
    //         // println!("{}", t.as_micros());
    //         display(self);
    //         sleep(t);
    //     }
    // }

    pub fn run(self: &mut Self) {
        // self.bus.memory[0xFFFF0] = 0xEA;
        // self.bus.memory[0xFFFF1] = 0x00;
        // self.bus.memory[0xFFFF2] = 0x00;
        // self.bus.memory[0xFFFF3] = 0x00;
        // self.bus.memory[0xFFFF4] = 0x00;

        // self.cpu.ax.set_x(0xC56D);
        // self.cpu.bx.set_x(0x1234);

        // self.bus.memory[0x00000] = 0xF7;
        // self.bus.memory[0x00001] = 0xFB;
        // self.bus.memory[0x00002] = 0xD4;
        // self.bus.memory[0x00003] = 0x0A;

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
    }
}
