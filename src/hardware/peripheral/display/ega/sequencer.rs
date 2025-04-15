#[derive(Default)]
pub struct Sequencer {
    pub address_reg: u8,
    pub reset_reg: u8,
    pub clocking_mode_reg: u8,
    pub map_mask_reg: u8,
    pub character_map_select_reg: u8,
    pub memory_mode_reg: u8,
}

impl Sequencer {
    pub fn write_reg(&mut self, port: usize, val: u8) {
        match port {
            0x3C4 => self.address_reg = val,
            0x3C5 => match self.address_reg {
                0 => self.reset_reg = val,
                1 => self.clocking_mode_reg = val,
                2 => self.map_mask_reg = val,
                3 => self.character_map_select_reg = val,
                4 => self.memory_mode_reg = val,
                _ => {}
            },
            _ => {}
        }
    }

    pub fn read_reg(&mut self, port: usize) -> u8 {
        match port {
            0x3C4 => self.address_reg,
            _ => 0
        }
    }
}
