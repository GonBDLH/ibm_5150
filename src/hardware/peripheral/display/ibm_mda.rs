use std::io::Read;

use crate::hardware::{peripheral::pic_8259::PIC8259, sys::ScreenMode};
use rayon::prelude::*;

use crate::hardware::peripheral::Peripheral;

use super::{
    crtc6845::{BlinkMode, CRTC6845},
    process_pixel_slice, BWChar, Character, DisplayAdapter,
};

// const IMG_SIZE: usize = 720 * 350;

#[allow(dead_code)]
pub struct IbmMDA {
    pub font_rom: Vec<u8>,

    font_map: [[[bool; 9]; 14]; 256],

    crtc: CRTC6845,

    cursor_color: u8,

    screen_dimensions: (usize, usize),
    char_dimensions: (usize, usize),
}

fn decode_font_map(font_rom: &[u8]) -> [[[bool; 9]; 14]; 256] {
    let mut font_map = [[[false; 9]; 14]; 256];

    for character in 0..255 {
        for row in 0..14 {
            let byte = if row < 8 {
                font_rom[row + character * 8]
            } else {
                font_rom[0x800 + (row - 8) + character * 8]
            };

            for col in 0..9 {
                let pixel = if col < 8 {
                    byte & (1 << (7 - col))
                } else if (0xC0..=0xDF).contains(&character) {
                    byte & 1
                } else {
                    0
                };

                font_map[character][row][col] = pixel > 0;
            }
        }
    }

    font_map
}

impl IbmMDA {
    pub fn new(screen_mode: ScreenMode) -> IbmMDA {
        // let a: Vec<u8> = (0..IMG_BUFF_SIZE).map(|x| if x % 4 == 3 {0xFF} else {0x00}).collect();
        // let a = vec![0x00; (dimensions.0 * dimensions.1 * 4.) as usize];
        let mut file =
            std::fs::File::open("roms/IBM_5788005_AM9264_1981_CGA_MDA_CARD.BIN").unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        let dimensions = screen_mode.get_pixel_dimensions();

        IbmMDA {
            font_rom: buf.clone(),
            font_map: decode_font_map(&buf),

            crtc: CRTC6845::default(),

            cursor_color: 0xFF,

            screen_dimensions: (dimensions.0 as usize, dimensions.1 as usize),
            char_dimensions: (9, 14),
        }
    }

    fn enabled(&self) -> bool {
        self.crtc.op1 & 0b00001000 > 0
    }
}

impl Default for IbmMDA {
    fn default() -> Self {
        IbmMDA::new(ScreenMode::MDA4025)
    }
}

fn decode_pixel_slice(
    font_map: &[[[bool; 9]; 14]; 256],
    row: usize,
    character: BWChar,
) -> [u8; 9 * 3] {
    let character_slice = font_map[character.index][row];
    let mut return_slice = [0x00; 9 * 3];

    process_pixel_slice(&mut return_slice, &character_slice, character);

    return_slice
}

impl DisplayAdapter for IbmMDA {
    fn create_frame(&mut self, vram: &[u8]) -> Vec<u8> {
        let mut img_buffer = vec![0x00; self.screen_dimensions.0 * self.screen_dimensions.1 * 3];

        if !self.enabled() {
            return img_buffer;
        }

        img_buffer
            .par_chunks_mut(9 * 3)
            .enumerate()
            .for_each(|(i, pixel_slice)| {
                let col_index = i % 80;
                let row_index = (i / 80) % 14;
                let row_char_index = i / (80 * 14);

                let char_index = (row_char_index * 80 + col_index) * 2;

                let vram_char = vram[char_index] as usize;
                let vram_attr = vram[char_index + 1];

                let character = BWChar::new(vram_char).decode_colors(vram_attr);

                let new_pixel_slice = decode_pixel_slice(&self.font_map, row_index, character);
                pixel_slice.copy_from_slice(&new_pixel_slice);
            });

        self.crtc.add_cursor(
            &mut img_buffer,
            self.screen_dimensions.0 / self.char_dimensions.0,
            self.char_dimensions,
            [0xFF, 0xFF, 0xFF],
        );

        img_buffer
    }

    fn inc_frame_counter(&mut self) {
        self.crtc.frame_counter += 1;
    }
}

impl Peripheral for IbmMDA {
    fn port_in(&mut self, port: u16) -> u16 {
        match port {
            0x3B8 => self.crtc.op1 as u16,
            // 0x3BA => self.crtc_sp as u16,
            0x3BA => {
                self.crtc.retrace = (self.crtc.retrace + 1) % 4;
                match self.crtc.retrace {
                    0 => 8,
                    1 => 0,
                    2 => 1,
                    3 => 0,
                    _ => unreachable!(),
                }
            }

            // 0x3B5 => self.crtc_registers[self.crtc_adddr_reg] as u16,
            0x3B5 => self.crtc.read_reg(self.crtc.adddr_reg) as u16,

            _ => 0, //TODO
        }
    }

    fn port_out(&mut self, val: u16, port: u16) {
        match port {
            0x3B8 => self.crtc.op1 = val as u8,
            0x3B4 => self.crtc.adddr_reg = (val as u8) as usize,
            // 0x3B5 =>  self.crtc_registers[self.crtc_adddr_reg] = val as u8,
            0x3B5 => self.crtc.reg_write(self.crtc.adddr_reg, val as u8),
            _ => {}
        }
    }

    fn update(&mut self, _pic: &mut PIC8259, _cycles: u32) {}
}
