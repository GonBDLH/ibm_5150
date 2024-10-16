use super::{pic_8259::PIC8259, Peripheral};

#[derive(Default, Clone, Copy)]
struct Channel {
    addr: u16,
    length: u16,

    toggle: bool,
}

pub struct DMA8237 {
    channels: [Channel; 4],
}

impl DMA8237 {
    pub fn new() -> Self {
        DMA8237 {
            channels: [Channel::default(); 4],
        }
    }

    fn write(&mut self, val: u16, channel: usize, opt: u16) {
        if !self.channels[channel].toggle {
            if opt == 0 {
                let addr = (self.channels[channel].addr & 0xFF00) | (val as u8) as u16;
                self.channels[channel].addr = addr;
            } else {
                let length = (self.channels[channel].length & 0xFF00) | (val as u8) as u16;
                self.channels[channel].length = length;
            }
        } else if opt == 0 {
            let addr = (self.channels[channel].addr & 0x00FF) | (val << 8);
            self.channels[channel].addr = addr;
        } else {
            let length = (self.channels[channel].length & 0x00FF) | (val & 0xFF00);
            self.channels[channel].length = length;
        }

        self.channels[channel].toggle = !self.channels[channel].toggle;
    }
}

impl Default for DMA8237 {
    fn default() -> Self {
        DMA8237::new()
    }
}

impl Peripheral for DMA8237 {
    fn port_in(&mut self, port: u16) -> u16 {
        if port <= 0x07 {
            let channel = port >> 1;
            let opt = port & 1;

            if opt == 0 {
                self.channels[channel as usize].addr
            } else {
                self.channels[channel as usize].length
            }
        } else {
            0
        }
    }

    fn port_out(&mut self, val: u16, port: u16) {
        if port <= 0x07 {
            let channel = port >> 1;
            let opt = port & 1;

            self.write(val, channel as usize, opt);
        } else {
        }
    }

    fn update(&mut self, _pic: &mut PIC8259, _cycles: u32) {}
}
