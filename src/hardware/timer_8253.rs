use super::peripheral::Peripheral;

#[derive(Clone, Copy)]
struct Channel {
    current_count: u16,
    reload_value: u16,
    latch_val: u16,

    toggle_read: bool,
    toggle_write: bool
}

impl Channel {
    fn new() -> Self {
        Self {
            current_count: 0,
            reload_value: 0,
            latch_val: 0,

            toggle_read: true,
            toggle_write: true,
        }
    }
}

pub struct TIM8253 {
    channels: [Channel; 3],
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
}

impl Peripheral for TIM8253 {
    fn port_in(&mut self, port: u16) -> u16 {
        match port {
            0x40..=0x42 => {
                let channel = ((self.mode_reg & 0b11000000) >> 6) as usize;
                let access_mode = (self.mode_reg & 0b00110000) >> 4;
                match access_mode {
                    0b00 => self.channels[channel].latch_val,
                    0b01 => self.channels[channel].current_count as u8 as u16,
                    0b10 => self.channels[channel].current_count >> 8,
                    0b11 => {
                        if self.channels[channel].toggle_read {
                            self.channels[channel].toggle_read = false;
                            self.channels[channel].current_count as u8 as u16
                        } else {
                            self.channels[channel].toggle_read = true;
                            self.channels[channel].current_count >> 8
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
                let channel = ((self.mode_reg & 0b11000000) >> 6) as usize;
                let access_mode = (self.mode_reg & 0b00110000) >> 4;
                match access_mode {
                    0b01 => self.channels[channel].current_count = (self.channels[channel].current_count & 0xFF00) | (val & 0x00FF),
                    0b10 => self.channels[channel].current_count = (self.channels[channel].current_count & 0x00FF) | ((val & 0x00FF) << 8),
                    0b11 => self.channels[channel].current_count = if self.channels[channel].toggle_write {
                        self.channels[channel].toggle_write = false;
                        (self.channels[channel].current_count & 0xFF00) | (val & 0x00FF)
                    } else {
                        self.channels[channel].toggle_write = true;
                        (self.channels[channel].current_count & 0x00FF) | ((val & 0x00FF) << 8)
                    },
                    _ => unreachable!()
                }
            },
            0x43 => self.mode_reg = val as u8,
            _ => unreachable!(),
        }
    }
}