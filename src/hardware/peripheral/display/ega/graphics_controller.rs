#[derive(Default)]
pub struct EgaGraphicsController {
    graphics_1_position_reg: u8,
    graphics_2_position_reg: u8,
    graphics_1_2_address_reg: u8,
    set_reset_reg: u8,
    enable_set_reset_reg: u8,
    color_compare_reg: u8,
    data_rotate_reg: u8,
    read_map_select_reg: u8,
    mode_register_reg: u8,
    miscellaneous_reg: u8,
    color_dont_care_reg: u8,
    bit_mask_reg: u8,
}

impl EgaGraphicsController {
    pub fn write_reg(&mut self, port: usize, val: u8) {
        match port {
            0x3CA => self.graphics_2_position_reg = val & 0b00000011,
            0x3CC => self.graphics_1_position_reg = val & 0b00000011,
            0x3CE => self.graphics_1_2_address_reg = val & 0b00001111,
            0x3CF => match self.graphics_1_2_address_reg {
                0x00 => self.set_reset_reg = val & 0b00001111,
                0x01 => self.color_compare_reg = val & 0b00001111,
                0x02 => self.enable_set_reset_reg = val & 0b00001111,
                0x03 => self.data_rotate_reg = val & 0b00011111,
                0x04 => self.read_map_select_reg = val & 0b00000111,
                0x05 => self.mode_register_reg = val & 0b00111111,
                0x06 => self.miscellaneous_reg = val & 0b00001111,
                0x07 => self.color_dont_care_reg = val & 0b00001111,
                0x08 => self.bit_mask_reg = val,
                _ => {},
            }
            _ => {}
        }
    }
}