use crate::hardware::cpu_8088::cpu_utils::*;
use crate::hardware::cpu_8088::instr_utils::{Length, Operand};
use crate::hardware::cpu_8088::CPU;

pub struct Bus {
    pub memory: [u8; 0x100000]
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            memory: [0x00; 0x100000]
            // memory: [0x00; 0x1000]
        }
    }

    pub fn read_8(self: &Self, segment: u16, offset: u16) -> u8 {
        let ea = ((segment as usize) << 4) + offset as usize;
        self.memory[ea % 0x100000]
    }

    pub fn read_16(self: &Self, segment: u16, offset: u16) -> u16 {
        to_u16(self.read_8(segment, offset), 
              self.read_8(segment, offset.wrapping_add(1)))
    }

    pub fn write_8(self: &mut Self, segment: u16, offset: u16, val: u8) {
        let ea = ((segment as usize) << 4) + offset as usize;
        self.memory[ea % 0x100000] = val;
    }

    pub fn write_16(self: &mut Self, segment: u16, offset: u16, val: u16) {
        self.write_8(segment, offset, val as u8);
        self.write_8(segment, offset.wrapping_add(1), (val >> 8) as u8);
    }

    pub fn write_length(&mut self, cpu: &mut CPU, length: Length, segment: Operand, offset: u16, val: u16) {
        let segment_u16 = cpu.get_segment(segment);

        match length {
            Length::Byte => self.write_8(segment_u16, offset, val as u8),
            Length::Word => self.write_16(segment_u16, offset, val),
            _ => unreachable!(),
        }
    }

    pub fn read_dir(self: &Self, dir: usize) -> u8 {
        self.memory[dir % 0x100000]
    }

    pub fn read_length(&self, cpu: &CPU, segment: Operand, offset: u16, length: Length) -> u16 {
        let segment_u16 = cpu.get_segment(segment);

        match length {
            Length::Byte => self.read_8(segment_u16, offset) as u16,
            Length::Word => self.read_16(segment_u16, offset),
            _ => unreachable!(),
        }
    }
}