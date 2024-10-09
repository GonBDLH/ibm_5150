// use std::fs::File;
use std::fs::File;

// use ggez::graphics::Image;
// use ggez::Context;

use super::bus::Bus;
use super::cpu_8088::{cpu_utils::get_address, CPU};
// use super::display::DisplayAdapter;
use super::peripheral::fdc_necupd765::FloppyDiskController;
use super::switches_cfg::*;

use std::fs::OpenOptions;

pub struct System {
    pub cpu: CPU,
    pub bus: Bus,

    // TODO REMOVE THIS
    pub disk_ctrl: FloppyDiskController,

    pub running: bool,

    pub file: File,
    cycles_step: u32,

    sw1: u8,
    sw2: u8,

    screen_dimensions: (f32, f32),
}

impl System {
    pub fn new(sw1: u8, sw2: u8, dimensions: (f32, f32)) -> Self {
        let sys = System {
            cpu: CPU::new(),
            bus: Bus::new(sw1, sw2, dimensions),
            disk_ctrl: FloppyDiskController::default(),

            running: false,

            file: OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open("logs/logs.txt")
                .unwrap(),
            cycles_step: 0,
            sw1,
            sw2,

            screen_dimensions: dimensions,
        };

        sys
    }
}

impl Default for System {
    fn default() -> Self {
        System::new(
            DD_ENABLE | RESERVED | MEM_64K | DISPLAY_MDA_80_25 | DRIVES_2,
            HIGH_NIBBLE | PLUS_0,
            (720., 350.),
        )
    }
}

use crate::{screen::DESIRED_FPS, util::debug_bios::debug_82};

impl System {
    pub fn rst(&mut self) {
        self.cpu = CPU::new();
        self.bus = Bus::new(self.sw1, self.sw2, self.screen_dimensions);

        self.running = false;
    }

    // Llamar cada frame
    pub fn update(&mut self, elapsed: f32) {
        let max_cycles = (4_772_726.7 * elapsed) as u32;
        let mut cycles_ran = 0;

        self.bus.ppi.key_input(&mut self.bus.pic);
        while cycles_ran <= max_cycles {
            if self.cpu.halted {
                print!("HALTED\r");
                cycles_ran += 1;
                continue;
            }

            self.step(&mut cycles_ran);
        }
    }

    pub fn update_debugger(&mut self) {
        let max_cycles = (4_772_726.7 / DESIRED_FPS) as u32;
        let mut cycles_ran = 0;

        while cycles_ran <= max_cycles {
            if self.cpu.halted {
                print!("HALTED\r");
                cycles_ran += 1;
                continue;
            }

            if !self.running || self.cpu.halted {
                break;
            }

            self.step(&mut cycles_ran);

            if get_address(&self.cpu) == 0x1A23 {
                self.running = false;
                break;
            }

            if get_address(&self.cpu) == 0x193B {
                self.running = false;
                break;
            }

            if get_address(&self.cpu) == 0x7C00 {
                self.running = false;
                break;
            }
        }
    }

    pub fn step(&mut self, cycles_ran: &mut u32) {
        // #[cfg(debug_assertions)]
        debug_82(&mut self.cpu);
        let (mut cycles, _ip) = self.cpu.fetch_decode_execute(&mut self.bus);
        // println!("{:04X}", _ip);

        self.cpu
            .handle_interrupts(&mut self.bus, &mut self.disk_ctrl, &mut cycles);

        self.cycles_step = cycles;
        // ACTUALIZAR PERIFERICOS
        self.bus.update_peripherals(cycles);

        *cycles_ran += cycles;
    }

    pub fn load_roms(&mut self) {
        // BASIC
        for (idx, element) in std::fs::read("roms/basic_1.10/IBM_5150-C1.10-U29-5000019.bin")
            .unwrap()
            .into_iter()
            .enumerate()
        {
            self.bus.memory[0xF6000 + idx] = element;
        }

        for (idx, element) in std::fs::read("roms/basic_1.10/IBM_5150-C1.10-U30-5000021.bin")
            .unwrap()
            .into_iter()
            .enumerate()
        {
            self.bus.memory[0xF8000 + idx] = element;
        }

        for (idx, element) in std::fs::read("roms/basic_1.10/IBM_5150-C1.10-U31-5000022.bin")
            .unwrap()
            .into_iter()
            .enumerate()
        {
            self.bus.memory[0xFA000 + idx] = element;
        }

        for (idx, element) in std::fs::read("roms/basic_1.10/IBM_5150-C1.10-U32-5000023.bin")
            .unwrap()
            .into_iter()
            .enumerate()
        {
            self.bus.memory[0xFC000 + idx] = element;
        }

        // BIOS
        for (idx, element) in std::fs::read("roms/BIOS_IBM5150_27OCT82_1501476_U33.BIN")
            .unwrap()
            .into_iter()
            .enumerate()
        {
            self.bus.memory[0xFE000 + idx] = element;
        }
    }

    pub fn load_test(&mut self, path: &str) {
        for (idx, element) in std::fs::read(path).unwrap().into_iter().enumerate() {
            self.cpu.cs = 0xF000;
            self.cpu.ip = 0xFFF0;
            self.bus.memory[0xF0000 + idx] = element;
        }
    }

    pub fn create_frame(&mut self) -> Vec<u8> {
        let vram = if self.sw1 & 0b00110000 == DISPLAY_MDA_80_25 {
            &self.bus.memory[0xB0000..0xB4000]
        } else {
            &self.bus.memory[0xB8000..0xBC000]
        };

        self.bus.display.inc_frame_counter();
        self.bus.display.create_frame(vram)
    }
}
