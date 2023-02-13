use super::Peripheral;

#[derive(Copy, Clone)]
pub struct PIC8259 {
    isr: u8,
    imr: u8,
    irr: u8,

    // max_prio: IRQs,

    icw: [u8; 4],
    icw_step: usize,
}

#[derive(Clone, Copy)]
pub enum IRQs {
    Irq0 = 0b00000001,
    Irq1 = 0b00000010,
    Irq2 = 0b00000100,
    Irq3 = 0b00001000,
    Irq4 = 0b00010000,
    Irq5 = 0b00100000,
    Irq6 = 0b01000000,
    Irq7 = 0b10000000,
}

impl PIC8259 {
    pub fn new() -> Self {
        Self { 
            isr: 0,                 // In-Service Register
            imr: 0xFF,              // Interrupt Mask Register
            irr: 0,                 // Interrupt Request Register

            // max_prio: IRQs::Irq0,

            icw: [0; 4],
            icw_step: 0,
        }
    }
}

impl PIC8259 {
    pub fn get_next(&mut self) -> u8 {
        let requested_ints = self.irr & !self.imr;
        for i in 0..8 {
            if requested_ints & 1 << (7 - i) > 0 {
                self.isr |= 1 << (7 - i);
                self.irr ^= i << (7 - i);
                return self.icw[1] + (7 - i);
            }
        }

        return 0;
    }

    pub fn has_int(&mut self) -> bool {
        (!self.imr & self.irr) > 0
    }

    pub fn irq(&mut self, irq: IRQs) {
        self.irr |= irq as u8;
    }
}

impl Peripheral for PIC8259 {
    fn port_in(&mut self, port: u16) -> u16 {
        if port == 0x20 {self.irr as u16} else {self.imr as u16}
    }

    fn port_out(&mut self, val: u16, port: u16) {
        let val = val as u8;
        
        if port == 0x20 {
            if val & 0x10 > 0 {
                // ICW1
                self.icw[self.icw_step] = val;
                self.imr = 0;
                self.icw_step += 1;
            }
            // TODO OCW
            if val & 0x20 > 0 {
                for i in 0..8 {
                    if self.isr & 1 << (7  - i) > 0 {
                        self.isr ^= 1 << (7  - i);
                    }
                }
            }
        } else {
            if self.icw_step == 1 {
                self.icw[self.icw_step] = val;
                self.icw_step += 1;
                if self.icw[0] & 0x02 == 1 {
                    self.icw_step += 1;
                }
            } else if self.icw_step < 4 {
                self.icw[self.icw_step] = val;
                self.icw_step += 1;
            } else {
                self.imr = val;
            }
        }
    }
}
