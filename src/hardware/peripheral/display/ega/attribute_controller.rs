#[derive(Default)]
pub struct EgaAttributeController {
    address_reg: u8,
    palette_registers: [u8; 0xF],
    mode_ctrl_reg: u8,
    overscan_color_reg: u8,
    color_plane_enable_reg: u8,
    horizontal_pel_panning_reg: u8,

    address_flip_flop: bool,
}

impl EgaAttributeController {
    pub fn write_reg(&mut self, port: usize, val: u8) {
        if port == 0x3C0 {
            // TODO IOR en 3BA o 3DA en el AttributeContoller
            self.address_flip_flop = !self.address_flip_flop;
            if self.address_flip_flop {
                self.address_reg = val & 0b00111111
            } else {
                match self.address_reg {
                    0x00..=0x0F => self.palette_registers[self.address_reg as usize] = val & 0b00111111,
                    0x10 => self.mode_ctrl_reg = val & 0b00001111,
                    0x11 => self.overscan_color_reg = val & 0b00111111,
                    0x12 => self.color_plane_enable_reg = val & 0b00111111,
                    0x13 => self.horizontal_pel_panning_reg = val & 0b00001111,
                    _ => {}
                }
            }
        }
    }
}
