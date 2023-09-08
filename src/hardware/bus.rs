use crate::hardware::cpu_8088::cpu_utils::*;
use crate::hardware::cpu_8088::instr_utils::Length;
use crate::hardware::cpu_8088::CPU;

use super::cpu_8088::instr_utils::Segment;
use super::display::ibm_mda::IbmMDA;
use super::peripheral::dma_8237::DMA8237;
use super::peripheral::pic_8259::PIC8259;
use super::peripheral::ppi_8255::PPI8255;
use super::peripheral::timer_8253::TIM8253;
use super::peripheral::Peripheral;

#[derive(Clone)]
pub struct Bus {
    // pub memory: [u8; 0x100000],
    pub memory: Vec<u8>,
    pub pic: PIC8259,
    pub pit: TIM8253,
    pub dma: DMA8237,
    pub ppi: PPI8255,
    pub mda: IbmMDA,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            memory: vec![0x00; 0x100000],
            pic: PIC8259::new(),
            pit: TIM8253::new(),
            dma: DMA8237::new(),
            ppi: PPI8255::new(),
            mda: IbmMDA::new(),
        }
    }

    pub fn update_peripherals(&mut self, cycles: u32) {
        self.update_timer();
        self.update_ppi(cycles);
    }

    fn update_timer(&mut self) {
        self.pit.update(&mut self.pic, &mut self.ppi);
    }

    fn update_ppi(&mut self, cycles: u32) {
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

            0x3B0..=0x3BF => self.mda.port_in(port),
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

            0x3B0..=0x3BF => self.mda.port_out(val, port),
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
        // #[cfg(not(feature = "tests"))] 
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
        Self::new()
    }
}
