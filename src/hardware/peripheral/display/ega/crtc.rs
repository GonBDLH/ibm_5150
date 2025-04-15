#[derive(Default)]
pub struct EgaCrtc {
    address_reg: u8,
    horizontal_total_reg: u8,
    horizontal_display_end_reg: u8,
    start_horizontal_blank_reg: u8,
    end_horizontal_blank_reg: u8,
    start_horizontal_retrace_reg: u8,
    end_horizontal_retrace_reg: u8,
    vertical_total_reg: u8,
    overflow_reg: u8,
    preset_row_scan_reg: u8,
    max_scan_line_reg: u8,
    cursor_start_reg: u8,
    cursor_end_reg: u8,
    start_address_high_reg: u8,
    start_address_low_reg: u8,
    cursor_location_high_reg: u8,
    cursor_location_low_reg: u8,
    vertical_retrace_start_reg: u8,
    light_pen_high_reg: u8,
    vertical_retrace_end_reg: u8,
    light_pen_low_reg: u8,
    vertical_display_end_reg: u8,
    offset_reg: u8,
    underline_location_reg: u8,
    start_vertical_blank_reg: u8,
    end_vertical_blank_reg: u8,
    mode_control_reg: u8,
    line_compare_reg: u8,
}

impl EgaCrtc {
    pub fn write_reg(&mut self, port: usize, val: u8) {
        match port {
            0x3B4 | 0x3D4 => self.address_reg = val & 0b00011111,
            0x3B5 | 0x3D5 => match self.address_reg {
                0x00 => self.horizontal_total_reg = val,
                0x01 => self.horizontal_display_end_reg = val,
                0x02 => self.start_horizontal_blank_reg = val,
                0x03 => self.end_horizontal_blank_reg = val & 0b01111111,
                0x04 => self.start_horizontal_retrace_reg = val,
                0x05 => self.end_horizontal_retrace_reg = val,
                0x06 => self.vertical_total_reg = val,
                0x07 => self.overflow_reg = val & 0b00111111,
                0x08 => self.preset_row_scan_reg = val & 0b00011111,
                0x09 => self.max_scan_line_reg = val & 0b00011111,
                0x0A => self.cursor_start_reg = val & 0b00011111,
                0x0B => self.cursor_end_reg = val & 0b01111111,
                0x0C => self.start_address_high_reg = val,
                0x0D => self.start_address_low_reg = val,
                0x0E => self.cursor_location_high_reg = val,
                0x0F => self.cursor_location_low_reg = val,
                0x10 => self.vertical_retrace_start_reg = val,
                0x11 => self.vertical_retrace_end_reg = val & 0b00111111,
                0x12 => self.vertical_display_end_reg = val,
                0x13 => self.offset_reg = val,
                0x14 => self.underline_location_reg = val & 0b00011111,
                0x15 => self.start_vertical_blank_reg = val,
                0x16 => self.end_vertical_blank_reg = val & 0b00011111,
                0x17 => self.mode_control_reg = val, // TODO igual aqui hacer funciones directamente como HW Reset
                0x18 => self.line_compare_reg = val,
                _ => {}
            },
            _ => {}
        }
    }

    fn read_reg(&self, port: usize) -> u8 {
        match port {
            0x3B4 | 0x3D4 => self.address_reg,
            0x3B5 | 0x3D5 => match self.address_reg {
                0x0C => self.start_address_high_reg,
                0x0D => self.start_address_low_reg,
                0x0E => self.cursor_location_high_reg,
                0x0F => self.cursor_location_low_reg,
                0x10 => self.light_pen_high_reg,
                0x11 => self.light_pen_low_reg,
                _ => 0,
            }
            _ => 0
        }
    } 
}
