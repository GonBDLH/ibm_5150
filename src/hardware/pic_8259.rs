use super::peripheral::Peripheral;

pub struct PIC8259 {
    isr: u8,
    imr: u8,
    irr: u8,

    max_prio_index: u8,
    icw: [u8; 4],
    icw_step: usize,
}

impl PIC8259 {
    pub fn new() -> Self {
        Self { 
            isr: 0,
            imr: 0,
            irr: 0,

            max_prio_index: 0,
            icw: [0; 4],
            icw_step: 0,
        }
    }
}

impl Peripheral for PIC8259 {
    fn port_in(&self, port: u16) -> u8 {
        if port == 0x20 {self.irr} else {self.imr}
    }

    // TODO No se si estara bien. Copiado de: https://github.com/NeatMonster/Intel8086/blob/master/src/fr/neatmonster/ibmpc/Intel8259.java
    fn port_out(&mut self, val: u16, port: u16) {
        if port == 0x20 {
            if val & 0x10 == 1 {                                    // ICW1
                self.imr = 0;
                self.max_prio_index = 7;
                self.icw[self.icw_step] = val as u8;
                self.icw_step += 1;
            } else {
                if val & 0x20 == 1 {                                // EOI
                    for i in 0..8 {
                        if self.isr >> i & 0b00000001 == 1 {
                            self.isr ^= 1 << i;
                        }
                    }
                }
            }
        } else {
            if self.icw_step == 1 {                                 // ICW2
                self.icw[self.icw_step] = val as u8;
                self.icw_step += 1;
                if self.icw[0] & 0x02 == 1 {self.icw_step += 1};
            } else if self.icw_step < 4 {                           // ICW3-4
                self.icw[self.icw_step] = val as u8;
                self.icw_step += 1;
            } else {
                self.imr = val as u8;                               // OCW1
            }
        }
    }

    fn is_connected(&self, port: u16) -> bool {
        port == 0x20 || port == 0x21
    }
}