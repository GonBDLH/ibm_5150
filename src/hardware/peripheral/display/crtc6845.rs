use std::default;

use crate::hardware::cpu_8088::cpu_utils::to_u16;

use super::DisplayAdapter;

#[derive(Default)]
pub struct CRTC6845 {
    horizontal_total_reg: u8,        // W
    horizontal_displayed_reg: u8,    // W
    horizontal_sync_pos_reg: u8,     // W
    sync_width_reg: u8,              // W
    vertical_total_reg: u8,          // W 7 bit
    vertical_total_adjust_reg: u8,   // W 5 bit
    vertical_displayed_reg: u8,      // W 7 bit
    vertical_sync_pos_reg: u8,       // W 7 bit
    interlace_mode_and_skew_reg: u8, // W 2 bit
    max_scan_line_address: u8,       // W 5 bit
    cursor_start_reg: u8,            // W 7 bit
    cursor_end_reg: u8,              // W 5 bit
    start_addressh_reg: u8,          // W 00XXXXXX
    start_addressl_reg: u8,          // W
    cursorh_reg: u8,                 // RW 00XXXXXX
    cursorl_reg: u8,                 // RW
    light_penh_reg: u8,              // R 00XXXXXX
    light_penl_reg: u8,              // R

    pub adddr_reg: usize,
    pub op1: u8,
    pub sp: u8,

    pub dirty_vram: bool,
    pub frame_counter: usize,

    pub retrace: u8,

    cursor_blink_state: BlinkState,
}

impl CRTC6845 {
    pub fn write_reg(&mut self, port: usize, val: u8) {
        match port {
            0 => self.horizontal_total_reg = val,
            1 => self.horizontal_displayed_reg = val,
            2 => self.horizontal_sync_pos_reg = val,
            3 => self.sync_width_reg = val,
            4 => self.vertical_total_reg = val & 0b01111111,
            5 => self.vertical_total_adjust_reg = val & 0b00011111,
            6 => self.vertical_displayed_reg = val & 0b01111111,
            7 => self.vertical_sync_pos_reg = val & 0b01111111,
            8 => self.interlace_mode_and_skew_reg = val & 0b00000011,
            9 => self.max_scan_line_address = val & 0b00011111,
            10 => self.cursor_start_reg = val & 0b01111111,
            11 => self.cursor_end_reg = val & 0b00011111,
            12 => self.start_addressh_reg = val & 0b00111111,
            13 => self.start_addressl_reg = val,
            14 => self.cursorh_reg = val & 0b00111111,
            15 => self.cursorl_reg = val,

            _ => {}
        }
    }

    pub fn read_reg(&mut self, port: usize) -> u8 {
        match port {
            14 => self.cursorh_reg,
            15 => self.cursorl_reg,
            16 => self.light_penh_reg,
            17 => self.light_penl_reg,

            _ => 0,
        }
    }

    pub fn get_cursor_xy(&self, screen_width: u16) -> (usize, usize) {
        let ch = self.cursorh_reg & 0b00111111;
        let cl = self.cursorl_reg;
        let cursor_addres = to_u16(cl, ch);

        let x = cursor_addres % screen_width;
        let y = cursor_addres / screen_width;

        (x as usize, y as usize)
    }

    pub fn get_cursor_start_end(&self) -> (usize, usize) {
        (
            (self.cursor_start_reg & 0b00011111) as usize,
            (self.cursor_end_reg & 0b00011111) as usize,
        )
    }

    pub fn get_cursor_blink(&self) -> BlinkMode {
        let blink_control = (self.cursor_start_reg & 0b01100000) >> 5;

        match blink_control {
            0b00 => BlinkMode::NonBlink,
            0b01 => BlinkMode::NonDisplay,
            0b10 => BlinkMode::Blink1_16,
            0b11 => BlinkMode::Blink1_32,
            _ => unreachable!(),
        }
    }

    pub fn add_cursor(
        &mut self,
        img_buffer: &mut [u8],
        screen_width: usize,
        char_dimensions: (usize, usize),
        cursor_color: [u8; 3],
    ) {
        let (x, y) = self.get_cursor_xy(screen_width as u16);
        let cursor_size = self.get_cursor_start_end();
        let blink_mode = self.get_cursor_blink();



        match blink_mode {
            BlinkMode::NonBlink => self.cursor_blink_state = BlinkState::Bright,
            BlinkMode::Blink1_16 => {
                if self.frame_counter % 16 == 0 {
                    self.frame_counter = 1;
                    self.cursor_blink_state = !self.cursor_blink_state;
                }
            }
            BlinkMode::Blink1_32 => {
                if self.frame_counter % 32 == 0 {
                    self.frame_counter = 1;
                    self.cursor_blink_state = !self.cursor_blink_state;
                }
            }
            BlinkMode::NonDisplay => return,
        };

        if self.cursor_blink_state == BlinkState::Dark {
            return;
        }

        for z in 0..char_dimensions.1 {
            for t in 0..char_dimensions.0 {
                if z < cursor_size.0 || z > cursor_size.1 {
                    continue;
                }

                let index = t
                    + x * char_dimensions.0
                    + z * char_dimensions.0 * screen_width
                    + y * char_dimensions.0 * screen_width * char_dimensions.1;

                img_buffer[index * 3..index * 3 + 3].copy_from_slice(&cursor_color);
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum BlinkMode {
    NonBlink,
    NonDisplay,
    Blink1_16,
    Blink1_32,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum BlinkState {
    Bright,
    #[default]
    Dark,
}

impl std::ops::Not for BlinkState {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Bright => Self::Dark,
            Self::Dark => Self::Bright,
        }
    }
}
