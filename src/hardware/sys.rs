// use std::fs::File;
use std::fs::File;

// use ggez::graphics::Image;
// use ggez::Context;

use super::bus::Bus;
use super::cpu_8088::{cpu_utils::get_address, CPU};
use super::peripheral::display::cga::CGA;
use super::peripheral::display::ibm_mda::IbmMDA;
use super::peripheral::display::DisplayAdapter;
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
}

impl System {
    pub fn new(sw1: u8, sw2: u8) -> Self {
        let sys = System {
            cpu: CPU::new(),
            bus: Bus::new(sw1, sw2),
            disk_ctrl: FloppyDiskController::default(),

            running: true,

            file: OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open("logs/logs.txt")
                .unwrap(),
            cycles_step: 0,
            sw1,
            sw2,
        };

        sys
    }
}

impl Default for System {
    fn default() -> Self {
        System::new(
            DD_ENABLE | RESERVED | MEM_64K | DISPLAY_MDA_80_25 | DRIVES_2,
            HIGH_NIBBLE | TOTAL_RAM_64,
        )
    }
}

use crate::util::debug_bios::debug_82;

impl System {
    pub fn rst(&mut self) {
        self.cpu = CPU::new();
        self.bus = Bus::new(self.sw1, self.sw2);

        // self.running = false;
    }

    pub fn update_sw(&mut self, sw1: u8, sw2: u8) {
        self.sw1 = sw1;
        self.sw2 = sw2;

        let screen_mode = ScreenMode::from_sw1(sw1);

        if screen_mode == ScreenMode::MDA8025 {
            self.bus.mda = Some(IbmMDA::new(screen_mode))
        } else if screen_mode == ScreenMode::CGA4025 || screen_mode == ScreenMode::CGA8025 {
            self.bus.cga = Some(CGA::new(screen_mode))
        }

        self.bus.ppi.sw1 = sw1;
        self.bus.ppi.sw2 = sw2;
    }

    // Llamar cada frame
    pub fn update(&mut self, elapsed: f32) {
        if !self.running {
            return;
        }

        let max_cycles = (4_772_726.7 * elapsed) as u32;
        let mut cycles_ran = 0;

        self.bus.ppi.key_input(&mut self.bus.pic);
        while cycles_ran <= max_cycles {
            self.step(&mut cycles_ran);
        }
    }

    // pub fn update_debugger(&mut self, elapsed: f32) {
    //     let max_cycles = (4_772_726.7 * elapsed) as u32;
    //     let mut cycles_ran = 0;

    //     while cycles_ran <= max_cycles {
    //         if self.cpu.halted {
    //             print!("HALTED\r");
    //             cycles_ran += 1;
    //             continue;
    //         }

    //         if !self.running || self.cpu.halted {
    //             break;
    //         }

    //         self.step(&mut cycles_ran);

    //         if get_address(&self.cpu) == 0x1A23 {
    //             self.running = false;
    //             break;
    //         }

    //         if get_address(&self.cpu) == 0x193B {
    //             self.running = false;
    //             break;
    //         }

    //         if get_address(&self.cpu) == 0x7C00 {
    //             self.running = false;
    //             break;
    //         }
    //     }
    // }

    pub fn step(&mut self, cycles_ran: &mut u32) {
        // #[cfg(debug_assertions)]
        debug_82(&mut self.cpu);
        let (mut cycles, _ip) = if !self.cpu.halted {
            self.cpu.fetch_decode_execute(&mut self.bus)
        } else {
            (1, self.cpu.ip)
        };
        // println!("{:04X}", _ip);

        self.cpu
            .handle_interrupts(&mut self.bus, &mut self.disk_ctrl, &mut cycles);

        self.cycles_step = cycles;
        // ACTUALIZAR PERIFERICOS
        self.bus.update_peripherals(cycles);

        *cycles_ran += cycles;
    }

    pub fn load_roms(&mut self, bios_selected: &BiosSelected) {
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

        let path = match bios_selected {
            BiosSelected::IbmPc => "roms/BIOS_IBM5150_27OCT82_1501476_U33.BIN",
            BiosSelected::GlaBios => "roms/GLABIOS_0.2.5_8P.ROM",
        };

        for (idx, element) in std::fs::read(path).unwrap().into_iter().enumerate() {
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

    // pub fn create_frame(&mut self) -> Vec<u8> {
    //     let vram = if self.sw1 & 0b00110000 == DISPLAY_MDA_80_25 {
    //         &self.bus.memory[0xB0000..0xB4000]
    //     } else {
    //         &self.bus.memory[0xB8000..0xBC000]
    //     };

    //     if let Some(display) = &mut self.bus.mda {
    //         display.inc_frame_counter();
    //         display.create_frame(vram)
    //     } else if let Some(display) = &mut self.bus.cga {
    //         display.inc_frame_counter();
    //         display.create_frame(vram)
    //     } else {
    //         let dimensions = self.screen_mode.get_pixel_dimensions();
    //         vec![0x00; dimensions.0 as usize * dimensions.1 as usize * 3]
    //     }
    // }

    pub fn create_mda_frame(&mut self) -> Vec<u8> {
        let screen_mode = ScreenMode::from_sw1(self.sw1);

        if screen_mode == ScreenMode::MDA8025 {
            let vram = &self.bus.memory[0xB0000..0xB4000];

            if let Some(display) = &mut self.bus.mda {
                display.inc_frame_counter();
                display.create_frame(vram)
            } else {
                let dimensions = screen_mode.get_pixel_dimensions();
                vec![0x00; dimensions.0 as usize * dimensions.1 as usize * 3]
            }
        } else {
            vec![0x00; 720 * 350 * 3]
        }
    }

    pub fn create_cga_40x25_frame(&mut self) -> Vec<u8> {
        let screen_mode = ScreenMode::from_sw1(self.sw1);

        if screen_mode == ScreenMode::CGA4025 {
            let vram = &self.bus.memory[0xB8000..0xBC000];

            if let Some(display) = &mut self.bus.cga {
                display.inc_frame_counter();
                display.create_frame(vram)
            } else {
                let dimensions = screen_mode.get_pixel_dimensions();
                vec![0x00; dimensions.0 as usize * dimensions.1 as usize * 3]
            }
        } else {
            vec![0x00; 320 * 200 * 3]
        }
    }

    pub fn create_cga_80x25_frame(&mut self) -> Vec<u8> {
        let screen_mode = ScreenMode::from_sw1(self.sw1);

        if screen_mode == ScreenMode::CGA8025 {
            let vram = &self.bus.memory[0xB8000..0xBC000];

            if let Some(display) = &mut self.bus.cga {
                display.inc_frame_counter();
                display.create_frame(vram)
            } else {
                let dimensions = screen_mode.get_pixel_dimensions();
                vec![0x00; dimensions.0 as usize * dimensions.1 as usize * 3]
            }
        } else {
            vec![0x00; 640 * 200 * 3]
        }
    }

    pub fn create_ega_320x200_16_frame(&mut self) -> Vec<u8> {
        // TODO
        
        vec![0x00; 320 * 200 * 3]
    }

    pub fn create_ega_640x350_16_frame(&mut self) -> Vec<u8> {
        // TODO
        
        vec![0x00; 640 * 350 * 3]
    }

    pub fn inser_floppy_disk(&mut self, path: &str, floppy: usize) {
        self.disk_ctrl.insert_disk(&mut self.bus, floppy, path);
    }

    pub fn eject_floppy_disk(&mut self, floppy: usize) {
        self.disk_ctrl.eject_disk(floppy);
    }

    pub fn key_down(&mut self, keycode: u8) {
        self.bus.ppi.key_down(keycode);
    }

    pub fn key_up(&mut self, keycode: u8) {
        self.bus.ppi.key_up(keycode);
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct ScreenMode {
    dimensions: (f32, f32),
    aspect_ratio: f32,
}

impl ScreenMode {
    pub const MDA8025: ScreenMode = ScreenMode {
        dimensions: (720., 350.),
        aspect_ratio: 1.333,
    };

    pub const CGA4025: ScreenMode = ScreenMode {
        dimensions: (320., 200.),
        aspect_ratio: 1.2,
    };

    pub const CGA8025: ScreenMode = ScreenMode {
        dimensions: (640., 200.),
        aspect_ratio: 2.4,
    };

    pub const EGA_320X200_16: ScreenMode = ScreenMode {
        dimensions: (320., 200.),
        aspect_ratio: 1.2,
    };

    pub const EGA_640X350_16: ScreenMode = ScreenMode {
        dimensions: (640., 350.),
        aspect_ratio: 1.37,
    };

    pub fn from_sw1(sw1: u8) -> Self {
        match sw1 & 0b00110000 {
            DISPLAY_MDA_80_25 => Self::MDA8025,
            DISPLAY_CGA_40_25 => Self::CGA4025,
            DISPLAY_CGA_80_25 => Self::CGA8025,
            DISPLAY_RESERVED => Self::MDA8025, // TODO
            _ => unreachable!(),
        }
    }
}

impl Default for ScreenMode {
    fn default() -> Self {
        ScreenMode::MDA8025
    }
}

impl ScreenMode {
    pub fn get_pixel_dimensions(&self) -> (f32, f32) {
        self.dimensions
    }

    pub fn get_aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }
}

#[derive(Default, PartialEq, Clone)]
pub enum BiosSelected {
    #[default]
    IbmPc,
    GlaBios,
}
