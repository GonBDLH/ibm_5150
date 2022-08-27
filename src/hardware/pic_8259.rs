use super::peripheral::Peripheral;

#[derive(Copy, Clone)]
pub struct PIC8259 {
    isr: u8,
    imr: u8,
    irr: u8,

    handled_int: u8,

    icw: [u8; 4],
    icw_step: usize,
}

impl PIC8259 {
    pub fn new() -> Self {
        Self { 
            isr: 0,
            imr: 0,
            irr: 0,

            handled_int: 0b1111000, // Solo interesan los 3 ultimos bits

            icw: [0; 4],
            icw_step: 0,
        }
    }
}

impl PIC8259 {
    fn get_next(&mut self) {
        for i in 0..8 {
            let mask = 1 << 7-i;
            if self.isr & mask > 0 {
                self.handled_int = 7 - i;
                return;
            }
        }
        self.handled_int = 0b1111000;
        return;
    }

    pub fn update(&mut self) -> (bool, u8) {
        (true, 0)
    }
}

impl Peripheral for PIC8259 {
    fn port_in(&mut self, port: u16) -> u16 {
        if port == 0x20 {self.irr as u16} else {self.imr as u16}
    }

    // TODO No se si estara bien. Copiado de: https://github.com/NeatMonster/Intel8086/blob/master/src/fr/neatmonster/ibmpc/Intel8259.java
    //                                  y de: https://github.com/Lichtso/DOS-Emulator/blob/master/src/pic.rs
    fn port_out(&mut self, val: u16, port: u16) {
        if port == 0x20 {
            if val & 0x10 == 1 {                                    // ICW1
                self.imr = 0;
                self.icw[self.icw_step] = val as u8;
                self.icw_step += 1;
            } else {
                if val & 0x20 == 1 {                                // Non Specific EOI
                    if self.handled_int & 0b00000111 > 0 {
                        let handled = self.handled_int & 0b0000111;
                        self.isr ^= 1 << handled;
                        self.get_next();
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
}