use std::fs::File;
use std::io::{stdout, Write};
use std::thread::sleep;
use std::time::{Instant, Duration};

use crossterm::execute;
use crossterm::terminal::SetSize;

use super::cpu_8088::CPU;
use super::bus::Bus;
use crate::util::debug::*;

pub struct System {
    pub cpu: CPU,
    pub bus: Bus,

    pub running: bool,

    pub not_sleeped: u64,
}

impl System {
    pub fn new() -> Self {
        execute!(stdout(), SetSize(120, 30)).unwrap();
        let sys = System { 
            cpu: CPU::new(),
            bus: Bus::new(),

            running: true,

            not_sleeped: 0,
        };
        
        sys
    }
}

impl System {
    fn sleep_time(&mut self, i: &mut u8, start: &mut Instant, cycles: &mut u32, file: &mut File) {
        let sum = i.overflowing_add(1);
        *i = sum.0;
        if sum.1 {
            let t = Duration::from(Instant::now() - *start);
            let to_sleep = Duration::new(0, *cycles * 209).checked_sub(t).unwrap_or_default();
            sleep(to_sleep);
    
            writeln!(file, "T. Dormido: {} - T. total ejecucion: {} - T. ejecución {}. Ips: {}", to_sleep.as_secs_f32(), Duration::new(0, *cycles * 209).as_secs_f32(), t.as_secs_f32(), u8::MAX as f32 / t.as_secs_f32()).unwrap();

            display(self);

            *start = Instant::now();
            *cycles = 0;
        }
    }

    pub fn clock_alt(self: &mut Self) {
        let mut i = 0u8;
        let mut cycles = 0;
        let mut file = File::create("logs/clock.txt").unwrap();

        let mut start = Instant::now();
        loop {
            cycles += self.cpu.fetch_decode_execute(&mut self.bus) as u32;
            

            self.sleep_time(&mut i, &mut start, &mut cycles, &mut file);
        }
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