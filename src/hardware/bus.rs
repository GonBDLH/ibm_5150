use super::cpu_utils::to_u16;

pub struct Bus {
    memory: [u8; 0x100000]
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            memory: [0x00; 0x100000]
        }
    }

    pub fn read_8(self: &Self, segment: u16, offset: u16) -> u8 {
        let ea = ((segment as usize) << 4) + offset as usize;
        self.memory[ea]
    }

    pub fn read_16(self: &Self, segment: u16, offset: u16) -> u16 {
        to_u16(self.read_8(segment, offset), 
              self.read_8(segment, offset + 1))
    }

    pub fn write_8(self: &mut Self, segment: u16, offset: u16, val: u8) {
        let ea = ((segment as usize) << 4) + offset as usize;
        self.memory[ea] = val;
    }

    pub fn write_16(self: &mut Self, segment: u16, offset: u16, val: u16) {
        self.write_8(segment, offset, (val >> 8) as u8);
        self.write_8(segment, offset + 1, val as u8);
    }

    pub fn read_dir(self: &Self, dir: usize) -> u8 {
        self.memory[dir]
    }
}