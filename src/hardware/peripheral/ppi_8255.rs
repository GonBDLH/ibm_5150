use super::Peripheral;

// IMPORTANTE: ESTAN AL REVES, LA POSICION 1 ES EL BIT 0.
//             ON = 0, OFF = 1
// const SW1: u8 = 0b00110000;
const SW1: u8 = 0b00110100;
// const SW2: u8 = 0b00001111;
const SW2: u8 = 0b11100000;

#[derive(Clone, Copy)]
pub struct PPI8255 {
    pub port_a: u8,
    port_b: u8,
    port_c: u8,
    mode_reg: u8,
}

impl PPI8255 {
    pub fn new() -> Self {
        PPI8255 { 
            port_a: 0x00,
            port_b: 0x00,
            port_c: 0x00,
            mode_reg: 0x00,
        }
    }
}

impl Peripheral for PPI8255 {
    fn port_in(&mut self, port: u16) -> u16 {
        let port = port & 0x3;
        match port {
            3 => self.mode_reg as u16,
            2 => self.read_pc() as u16,
            1 => self.port_b as u16,
            0 => self.read_pa() as u16,
            _ => unreachable!(),
        }    
    }

    fn port_out(&mut self, val: u16, port: u16) {
        // TODO
        let port = port & 0x3;
        match port {
            3 => self.mode_reg = val as u8,
            2 => self.port_c = val as u8,
            1 => self.port_b = val as u8,
            0 => self.port_a = val as u8,
            _ => unreachable!(),
        };
    }
}

impl PPI8255 {
    fn read_pa(&mut self) -> u8 {
        if self.port_b & 0x80 == 0x80 {
            SW1
        } else {
            // TODO KEY_CODE
            self.port_a
        }
    }

    fn read_pc(&mut self) -> u8 {
        if self.port_b & 0x04 == 0x04 {
            SW2 & 0x0F | self.port_c & 0xF0
        } else {
            (SW2 >> 4) & 0x01 | self.port_c & 0xF0
        }
    }
}
