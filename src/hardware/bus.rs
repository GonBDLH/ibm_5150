use std::borrow::BorrowMut;
use std::sync::{Arc, Mutex};

use rayon::prelude::ParallelIterator;

use crate::hardware::cpu_8088::cpu_utils::*;
use crate::hardware::cpu_8088::instr_utils::Length;
use crate::hardware::cpu_8088::CPU;
use crate::hardware::sys::ScreenMode;

use super::cpu_8088::instr_utils::Segment;
use super::peripheral::display::cga::CGA;
use super::peripheral::display::ega::EGA;
use super::peripheral::display::ibm_mda::IbmMDA;
use super::peripheral::display::DisplayAdapter;
use super::peripheral::dma_8237::DMA8237;
use super::peripheral::fdc_necupd765::FloppyDiskController;
use super::peripheral::pic_8259::PIC8259;
use super::peripheral::pit_8253::TIM8253;
use super::peripheral::ppi_8255::PPI8255;
use super::peripheral::Peripheral;
use super::switches_cfg::*;

pub trait DisplayPeripheral: DisplayAdapter + Peripheral {}

impl DisplayPeripheral for CGA {}
impl DisplayPeripheral for IbmMDA {}
impl DisplayPeripheral for EGA {}

pub struct Bus {
    // pub memory: [u8; 0x100000],
    pub memory: Vec<u8>,
    pub pic: PIC8259,
    pub pit: TIM8253,
    pub dma: DMA8237,
    pub ppi: PPI8255,
    pub mda: Option<IbmMDA>,
    pub cga: Option<CGA>,
    pub ega: Option<EGA>, // TODO Las combinaciones pueden ser MDA/EGA-CGA o MDA-EGA/CGA
    pub fdc: FloppyDiskController,
}

impl Bus {
    pub fn new(sw1: u8, sw2: u8) -> Self {
        if sw1 & 0b00110000 == DISPLAY_MDA_80_25 {
            Bus {
                memory: vec![0x00; 0x100000],
                pic: PIC8259::new(),
                pit: TIM8253::new(),
                dma: DMA8237::new(),
                ppi: PPI8255::new(sw1, sw2),
                // display: Box::new(IbmMDA::new(screen_mode)),
                mda: Some(IbmMDA::new(ScreenMode::from_sw1(sw1))),
                cga: None,
                ega: None,
                fdc: FloppyDiskController::default(),
            }
        } else if sw1 & 0b00110000 == DISPLAY_CGA_40_25 || sw1 & 0b00110000 == DISPLAY_CGA_80_25 {
            Bus {
                memory: vec![0x00; 0x100000],
                pic: PIC8259::new(),
                pit: TIM8253::new(),
                dma: DMA8237::new(),
                ppi: PPI8255::new(sw1, sw2),
                mda: None,
                cga: Some(CGA::new(ScreenMode::from_sw1(sw1))),
                ega: None,
                fdc: FloppyDiskController::default(),
            }
        } else {
            Bus {
                memory: vec![0x00; 0x100000],
                pic: PIC8259::new(),
                pit: TIM8253::new(),
                dma: DMA8237::new(),
                ppi: PPI8255::new(sw1, sw2),
                mda: None,
                cga: None,
                ega: Some(EGA::new()),
                fdc: FloppyDiskController::default(),
            }
        }
    }

    pub fn update_peripherals(&mut self, cycles: u32) {
        self.pit.update(&mut self.pic, cycles);
        self.ppi.update(&mut self.pic, cycles);
    }

    pub fn port_in(&mut self, port: u16) -> u16 {
        match port {
            0x00..=0x0F => self.dma.port_in(port),
            0x20..=0x21 => self.pic.port_in(port),
            0x40..=0x43 => self.pit.port_in(port),
            0x60..=0x63 => self.ppi.port_in(port),
            0x80..=0x83 => {
                /* TODO Reg pagina DMA */
                0
            }
            0xA0..=0xAF => 0,

            // TODO QUIZAS AQUI HAY QUE CAMBIAR ALGO, EGA SOLO LEE EN 0x3B2
            0x3B0..=0x3BF => match (&mut self.mda, &mut self.ega) {
                (None, None) => 0,
                (Some(mda), None) => mda.port_in(port),
                (None, Some(ega)) => ega.port_in(port),
                (Some(mda), Some(_)) => mda.port_in(port),
            },
            0x3C0..=0x3CF => {
                if let Some(ega) = &mut self.ega {
                    ega.port_in(port)
                } else {
                    0
                }
            }
            // TODO QUIZAS AQUI HAY QUE CAMBIAR ALGO, EGA SOLO LEE EN 0x3D2
            0x3D0..=0x3DF => match (&mut self.cga, &mut self.ega) {
                (None, None) => 0,
                (Some(cga), None) => cga.port_in(port),
                (None, Some(ega)) => ega.port_in(port),
                (Some(cga), Some(_)) => cga.port_in(port),
            },
            0x3F0..=0x3F7 => self.fdc.port_in(port),
            _ => 0,
        }
    }

    pub fn port_out(&mut self, cpu: &mut CPU, val: u16, port: u16) {
        match port {
            0x00..=0x0F => self.dma.port_out(val, port),
            0x20..=0x21 => self.pic.port_out(val, port),
            0x40..=0x43 => self.pit.port_out(val, port),
            0x60..=0x63 => self.ppi.port_out(val, port),
            0x80..=0x83 => { /* TODO Reg pagina DMA */ }
            0xA0..=0xAF => cpu.nmi_out(val),

            // TODO QUIZAS AQUI HAY QUE CAMBIAR ALGO, EGA SOLO ESCRIBE EN 0x3B2
            0x3B0..=0x3BF => match (&mut self.mda, &mut self.ega) {
                (None, None) => {},
                (Some(mda), None) => mda.port_out(val, port),
                (None, Some(ega)) => ega.port_out(val, port),
                (Some(mda), Some(_)) => mda.port_out(val, port),
            },
            0x3C0..=0x3CF => {
                if let Some(ega) = &mut self.ega {
                    ega.port_out(val, port)
                }
            }
            // TODO QUIZAS AQUI HAY QUE CAMBIAR ALGO, EGA SOLO ESCRIBE EN 0x3D2
            0x3D0..=0x3DF => match (&mut self.cga, &mut self.ega) {
                (None, None) => {},
                (Some(cga), None) => cga.port_out(val, port),
                (None, Some(ega)) => ega.port_out(val, port),
                (Some(cga), Some(_)) => cga.port_out(val, port),
            },
            0x3F0..=0x3F7 => self.fdc.port_out(val, port),
            _ => {}
        };
    }

    pub fn read_8(&self, segment: u16, offset: u16) -> u8 {
        let ea = (((segment as usize) * 0x10) + offset as usize) % 0x100000;

        self.memory[ea]
    }

    pub fn read_16(&self, segment: u16, offset: u16) -> u16 {
        to_u16(
            self.read_8(segment, offset),
            self.read_8(segment, offset.wrapping_add(1)),
        )
    }

    pub fn write_8(&mut self, segment: u16, offset: u16, val: u8) {
        let ea = (((segment as usize) * 0x10) + offset as usize) % 0x100000;

        // NO ESCRIBIR EN ROM
        #[cfg(not(test))]
        if ea >= 0xC0000 {
            return;
        }

        self.memory[ea] = val;
    }

    pub fn write_16(&mut self, segment: u16, offset: u16, val: u16) {
        self.write_8(segment, offset, val as u8);
        self.write_8(segment, offset.wrapping_add(1), (val >> 8) as u8);
    }

    pub fn write_length(
        &mut self,
        cpu: &mut CPU,
        length: Length,
        segment: Segment,
        offset: u16,
        val: u16,
    ) {
        let segment_u16 = cpu.get_segment(segment);

        match length {
            Length::Byte => self.write_8(segment_u16, offset, val as u8),
            Length::Word => self.write_16(segment_u16, offset, val),
            _ => unreachable!(),
        }
    }

    pub fn read_dir(&self, dir: usize) -> u8 {
        self.memory[dir % 0x100000]
    }

    pub fn read_length(&self, cpu: &CPU, segment: Segment, offset: u16, length: Length) -> u16 {
        let segment_u16 = cpu.get_segment(segment);

        match length {
            Length::Byte => self.read_8(segment_u16, offset) as u16,
            Length::Word => self.read_16(segment_u16, offset),
            _ => unreachable!(),
        }
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self::new(
            DD_ENABLE | RESERVED | MEM_64K | DISPLAY_MDA_80_25 | DRIVES_2,
            HIGH_NIBBLE | TOTAL_RAM_64,
        )
    }
}
