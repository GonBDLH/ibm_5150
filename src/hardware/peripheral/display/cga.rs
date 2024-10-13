use std::{io::Read, mem::transmute};

use crate::{frontend::ScreenMode, hardware::peripheral::pic_8259::PIC8259};
use rand::{thread_rng, Rng};
use rayon::{
    prelude::{IndexedParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

use crate::hardware::peripheral::Peripheral;

use super::{crtc6845::CRTC6845, process_pixel_slice, Character, ColorChar, DisplayAdapter};

pub struct CGA {
    font_map: [[[bool; 8]; 8]; 256],

    crtc: CRTC6845,

    screen_mode: ScreenMode,
    char_dimensions: (usize, usize),

    color: u8,
}

impl CGA {
    pub fn new(screen_mode: ScreenMode) -> Self {
        let mut file =
            std::fs::File::open("roms/IBM_5788005_AM9264_1981_CGA_MDA_CARD.BIN").unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();

        Self {
            font_map: decode_font_map(&buf[0x1800..]),
            crtc: CRTC6845::default(),
            screen_mode,
            char_dimensions: (8, 8),

            color: 0,
        }
    }

    fn enabled(&self) -> bool {
        self.crtc.op1 & 0b00001000 > 0
    }

    fn alphanumeric_mode(&mut self, vram: &[u8], img_buffer: &mut [u8]) {
        let dimensions = self.screen_mode.get_pixel_dimensions();

        let screen_character_width = dimensions.0 as usize / self.char_dimensions.0;

        img_buffer
            .par_chunks_mut(8 * 3)
            .enumerate()
            .for_each(|(i, pixel_slice)| {
                let col_index = i % screen_character_width;
                let row_index = (i / screen_character_width) % 8;
                let row_char_index = i / (screen_character_width * 8);

                let char_index = (row_char_index * screen_character_width + col_index) * 2;

                let vram_char = vram[char_index] as usize;
                let vram_attr = vram[char_index + 1];

                let character = ColorChar::new(vram_char).decode_colors(vram_attr);

                let new_pixel_slice = decode_pixel_slice(&self.font_map, row_index, character);
                pixel_slice.copy_from_slice(&new_pixel_slice);
            });
        // self.add_cursor();
    }

    #[inline]
    fn medium_res_mode(&self, pixel_slice: &mut [u8], row_slice: &[u8]) {
        let palette = (self.color >> 5) & 1;

        for (group_index, pixel_group) in row_slice.iter().enumerate() {
            for pixel_offset in 0..4 {
                let pixel = pixel_group >> (2 * (3 - pixel_offset)) & 3;
                let color_bytes = [0xAA * (pixel >> 1), 0xAA * (pixel & 1), 0xAA * palette];

                let pixel_slice_start = group_index * 12 + pixel_offset * 3;
                let pixel_slice_end = pixel_slice_start + 3;

                pixel_slice[pixel_slice_start..pixel_slice_end].copy_from_slice(&color_bytes)
            }
        }
    }

    #[inline]
    fn high_res_mode(&self, pixel_slice: &mut [u8], row_slice: &[u8]) {
        for (group_index, pixel_group) in row_slice.iter().enumerate() {
            for pixel_offset in 0..8 {
                let pixel = (pixel_group >> (7 - pixel_offset)) & 1;

                let color = 0xFFFFFFFFu32 * pixel as u32;

                let color_bytes = color.to_be_bytes();
                let pixel_slice_start = group_index * 24 + pixel_offset * 3;
                let pixel_slice_end = pixel_slice_start + 3;

                pixel_slice[pixel_slice_start..pixel_slice_end].copy_from_slice(&color_bytes[0..3])
            }
        }
    }

    fn graphic_mode(&mut self, vram: &[u8], img_buffer: &mut [u8]) {
        let dimensions = self.screen_mode.get_pixel_dimensions();

        let scaling = if let ScreenMode::CGA8025 = self.screen_mode {
            2
        } else {
            1
        };

        img_buffer
            .par_chunks_mut(dimensions.0 as usize * 3)
            .enumerate()
            .for_each(|(i, pixel_slice)| {
                let row_slice = if i % 2 == 0 {
                    let slice_start = i * dimensions.0 as usize / (8 * scaling);
                    let slice_end = slice_start + dimensions.0 as usize / (4 * scaling);
                    &vram[slice_start..slice_end]
                } else {
                    let slice_start = (i - 1) * dimensions.0 as usize / (8 * scaling);
                    let slice_end = slice_start + dimensions.0 as usize / (4 * scaling);
                    &vram[(0x2000 + slice_start)..(0x2000 + slice_end)]
                };

                // TODO No me gusta como esta esto
                if scaling == 2 {
                    self.high_res_mode(pixel_slice, row_slice);
                } else {
                    self.medium_res_mode(pixel_slice, row_slice);
                }
            });
    }
}

fn decode_font_map(font_rom: &[u8]) -> [[[bool; 8]; 8]; 256] {
    let mut font_map = [[[false; 8]; 8]; 256];

    for character in 0..255 {
        for row in 0..8 {
            let byte = font_rom[row + character * 8];

            for col in 0..8 {
                let pixel = byte & (1 << (7 - col));

                font_map[character][row][col] = pixel > 0;
            }
        }
    }

    font_map
}

fn decode_pixel_slice(
    font_map: &[[[bool; 8]; 8]; 256],
    row: usize,
    character: ColorChar,
) -> [u8; 8 * 3] {
    let character_slice = font_map[character.index][row];
    let mut return_slice = [0xFF; 8 * 3];

    process_pixel_slice(&mut return_slice, &character_slice, character);

    return_slice
}

impl Peripheral for CGA {
    fn port_in(&mut self, port: u16) -> u16 {
        match port {
            0x3D8 => self.crtc.op1 as u16,
            0x3DA => {
                self.crtc.retrace = (self.crtc.retrace + 1) % 4;

                match self.crtc.retrace {
                    0 => 8,
                    1 => 0,
                    2 => 1,
                    3 => 0,
                    _ => unreachable!(),
                }
            }

            0x3D5 => self.crtc.read_reg(self.crtc.adddr_reg) as u16,

            _ => 0, //TODO
        }
    }

    fn port_out(&mut self, val: u16, port: u16) {
        match port {
            0x3D9 => self.color = val as u8,
            0x3D8 => self.crtc.op1 = val as u8,
            0x3D5 => self.crtc.reg_write(self.crtc.adddr_reg, val as u8),
            0x3D4 => self.crtc.adddr_reg = (val as u8) as usize,

            _ => {}
        }
    }

    fn update(&mut self, _pic: &mut PIC8259, _cycles: u32) {
        
    }
}

impl DisplayAdapter for CGA {
    fn create_frame(&mut self, vram: &[u8]) -> Vec<u8> {
        let dimensions = self.screen_mode.get_pixel_dimensions();

        let mut img_buffer = vec![0x00; dimensions.0 as usize * dimensions.1 as usize * 3];

        if !self.enabled() {
            return img_buffer;
        }

        if self.crtc.op1 & 0b00000010 > 0 {
            self.graphic_mode(vram, &mut img_buffer);
        } else {
            self.alphanumeric_mode(vram, &mut img_buffer);
        }

        self.crtc.add_cursor(
            &mut img_buffer,
            dimensions.0 as usize / self.char_dimensions.0,
            self.char_dimensions,
            [0xAA, 0xAA, 0xAA],
        );

        img_buffer
    }

    fn inc_frame_counter(&mut self) {
        self.crtc.frame_counter += 1;
    }
}
