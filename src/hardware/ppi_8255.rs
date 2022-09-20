use super::peripheral::Peripheral;

#[derive(Clone, Copy)]
pub struct PPI8255 {
    ports: [u8; 3],
    mode_reg: u8,
}

impl PPI8255 {
    pub fn new() -> Self {
        PPI8255 { 
            ports: [0x00; 3],
            mode_reg: 0x00 
        }
    }
}

impl Peripheral for PPI8255 {
    fn port_in(&mut self, port: u16) -> u16 {
        let port = port & 0x3;
        match port {
            3 => self.mode_reg as u16,
            _ => self.ports[port as usize] as u16,
        }    
    }

    fn port_out(&mut self, val: u16, port: u16) {
        // TODO
        let port = port & 0x3;
        match port {
            3 => self.mode_reg = val as u8,
            _ => self.ports[port as usize] = val as u8,
        };
    }
}
