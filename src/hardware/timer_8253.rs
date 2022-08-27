use super::{peripheral::Peripheral, pic_8259::PIC8259};

#[derive(Clone, Copy)]
pub struct Channel {
    pub current_count: u32,
    reload_value: u16,
    latch_val: u16,
    rl_mode: u8,
    mode: u8,

    toggle: bool,
}

impl Channel {
    fn new() -> Self {
        Self {
            current_count: 0,
            reload_value: 0,
            latch_val: 0,
            rl_mode: 0,
            mode: 0,

            toggle: true,
        }
    }
}

#[derive(Clone, Copy)]
pub struct TIM8253 {
    pub channels: [Channel; 3],
    mode_reg: u8,
}

impl TIM8253 {
    pub fn new() -> Self {
        Self {
            channels: [Channel::new(); 3],
            mode_reg: 0,
        }
    }

    // TODO FUNCIONAMIENTO
    pub fn update(&mut self, cycles: u32, pic: &mut PIC8259) {
        let before = self.get_current_count(0);
        self.channels[0].current_count = (self.channels[0].current_count & 0x3FFF).wrapping_sub(cycles);
        let after = self.get_current_count(0); 

        if after == 0 || after > before {
            // pic.
        }

        // self.channels[0].current_count -= cycles;
        // if self.channels[0].timer_cycles >= 0x1FFFF {
        //     self.channels[0].current_count = self.channels[0].current_count.wrapping_sub(1);
        //     self.channels[0].timer_cycles >>= 18;
        // }
        // if self.channels[0].current_count == 0 {
        //     // TODO INT
            
        // }
    }
}

impl Peripheral for TIM8253 {
    fn port_in(&mut self, port: u16) -> u16 {
        match port {
            0x40..=0x42 => {
                let channel = (port & 0b11) as usize;
                let access_mode = (self.channels[channel].mode & 0b00110000) >> 4;

                match access_mode {
                    0b00 => self.channels[channel].latch_val,
                    0b01 => self.get_current_count(channel) as u8 as u16,
                    0b10 => self.get_current_count(channel) >> 8,
                    0b11 => {
                        if self.channels[channel].toggle {
                            self.channels[channel].toggle = false;
                            self.get_current_count(channel) as u8 as u16
                        } else {
                            self.channels[channel].toggle = true;
                            self.get_current_count(channel) >> 8
                        }
                    }
                    _ => unreachable!()
                }
            }, // TODO
            _ => 0
        }
    }

    fn port_out(&mut self, val: u16, port: u16) {
        match port {
            0x40..=0x42 => {
                let channel = (port & 0b11) as usize;
                self.set_current_count(channel, val)
            },
            0x43 => {
                self.mode_reg = val as u8;
                let channel = ((self.mode_reg & 0b11000000) >> 6) as usize;
                let access_mode = (self.mode_reg & 0b00110000) >> 4;
                match access_mode {
                    0b00 => self.channels[channel].latch_val = self.get_current_count(channel),
                    _ => {self.channels[channel].rl_mode = access_mode},
                }
            },
            _ => unreachable!(),
        }
    }
}

impl TIM8253 {
    fn set_current_count(&mut self, channel: usize, val: u16) {
        match self.channels[channel].rl_mode {
            0b01 => self.channels[channel].current_count = (((self.get_current_count(channel) & 0xFF00) | (val & 0x00FF)) as u32) << 2,
            0b10 => self.channels[channel].current_count = (((self.get_current_count(channel) & 0x00FF) | ((val & 0x00FF) << 8)) as u32) << 2,
            0b11 => self.channels[channel].current_count = if self.channels[channel].toggle {
                self.channels[channel].toggle = false;
                (((self.get_current_count(channel) & 0xFF00) | (val & 0x00FF)) as u32) << 2
            } else {
                self.channels[channel].toggle = true;
                (((self.get_current_count(channel) & 0x00FF) | ((val & 0x00FF) << 8)) as u32) << 2
            },
            _ => unreachable!()
        }
    }

    fn get_current_count(&self, channel: usize) -> u16 {
        (self.channels[channel].current_count >> 2) as u16
    }
}
