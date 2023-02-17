// use std::fs::File;
use std::fs::File;

use super::cpu_8088::CPU;
use super::bus::Bus;

use std::fs::OpenOptions;

pub struct System {
    pub cpu: CPU,
    pub bus: Bus,

    pub running: bool,

    pub file: File,
    cycles_step: u32,
}

impl System {
    pub fn new() -> Self {
        let sys = System { 
            cpu: CPU::new(),
            bus: Bus::new(),

            running: false,

            file: OpenOptions::new().create(true).write(true).open("logs/logs.txt").unwrap(),
            cycles_step: 0,
        };
      
        sys
    }
}

use crate::{DESIRED_FPS, util::debug_bios::debug_82};

impl System {
    pub fn rst(&mut self) {
        self.cpu = CPU::new();
        self.bus = Bus::new();

        self.running = false;
    }

    // Llamar cada frame
    pub fn update(&mut self) {
        let max_cycles = (4_772_726.7 / DESIRED_FPS) as u32;
        let mut cycles_ran = 0;

        while cycles_ran <= max_cycles {
            if self.cpu.halted {
                print!("HALTED\r");
                cycles_ran += 1;
                continue;
            }
            self.step(&mut cycles_ran);
        }
    }

    #[inline]
    pub fn step(&mut self, cycles_ran: &mut u32) {
        // if self.cpu.ip == 0xE287 {
        //     let _a = 0;
        // }
        
        debug_82(&mut self.cpu);
        let (cycles, _ip) = self.cpu.fetch_decode_execute(&mut self.bus);
        self.cycles_step = cycles;
        // println!("{:04X}", _ip);
        
        self.cpu.handle_interrupts(&mut self.bus);
        
        // ACTUALIZAR PERIFERICOS
        self.bus.update_peripherals(cycles);

        *cycles_ran += cycles;
    }

    pub fn load_roms(&mut self) {
        // BASIC
        for (idx, element) in std::fs::read("roms/basic_1.10/IBM_5150-C1.10-U29-5000019.bin").unwrap().into_iter().enumerate() {
            self.bus.memory[0xF6000 + idx] = element;
        }

        for (idx, element) in std::fs::read("roms/basic_1.10/IBM_5150-C1.10-U30-5000021.bin").unwrap().into_iter().enumerate() {
            self.bus.memory[0xF8000 + idx] = element;
        }
        
        for (idx, element) in std::fs::read("roms/basic_1.10/IBM_5150-C1.10-U31-5000022.bin").unwrap().into_iter().enumerate() {
            self.bus.memory[0xFA000 + idx] = element;
        }
        
        for (idx, element) in std::fs::read("roms/basic_1.10/IBM_5150-C1.10-U32-5000023.bin").unwrap().into_iter().enumerate() {
            self.bus.memory[0xFC000 + idx] = element;
        }
        
        // BIOS
        for (idx, element) in std::fs::read("roms/BIOS_IBM5150_27OCT82_1501476_U33.BIN").unwrap().into_iter().enumerate() {
            self.bus.memory[0xFE000 + idx] = element;
        }
    }
}