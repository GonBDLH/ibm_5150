use std::{io::Read, mem::transmute};

use crate::hardware::display::process_pixel_slice;
use ggez::graphics::{Color, Image, ImageFormat};
use rayon::{
    prelude::{IndexedParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

use crate::hardware::peripheral::Peripheral;

use super::{crtc6845::CRTC6845, Character, ColorChar, DisplayAdapter};

const PALETTE: [[u32; 4]; 4] = [[0x000000FF, 0x00AA00FF, 0xAA0000FF, 0xAA5500FF],       // PALETTE_0_LOW_INTENSITY
                                [0x555555FF, 0x55FF55FF, 0xFF5555FF, 0xFFFF55FF],       // PALETTE_0_HIGH_INTENSITY
                                [0x000000FF, 0x0000AAFF, 0xAA00AAFF, 0xAAAAAAFF],       // PALETTE_1_LOW_INTENSITY
                                [0x555555FF, 0x55FFFFFF, 0xFF55FFFF, 0xFFFFFFFF]];      // PALETTE_1_HIGH_INTENSITY

// const PALETTE: [[u32; 4]; 4] = [[0x000000FF, 0x00AAAAFF, 0xAA00AAFF, 0xAAAAAAFF],       // PALETTE_0_LOW_INTENSITY
//                                 [0x555555FF, 0x55FFFFFF, 0xFF55FFFF, 0xFFFFFFFF],       // PALETTE_0_HIGH_INTENSITY
//                                 [0x000000FF, 0x00AA00FF, 0xAA0000FF, 0xAA5500FF],       // PALETTE_1_LOW_INTENSITY
//                                 [0x555555FF, 0x55FF55FF, 0xFF5555FF, 0xFFFF55FF]];      // PALETTE_1_HIGH_INTENSITY

const FULL_PALETTE: [u32; 16] = [0x000000FF, 0x555555FF, 0x0000AAFF, 0x5555FFFF,
                                 0x00AA00FF, 0x55FF55FF, 0x00AAAAFF, 0x55FFFFFF,
                                 0xAA0000FF, 0xFF5555FF, 0xAA00AAFF, 0xFF55FFFF,
                                 0xAA5500FF, 0xFFFF55FF, 0xAAAAAAFF, 0xFFFFFFFF];

pub struct CGA {
    pub img_buffer: Vec<u8>,

    font_map: [[[bool; 8]; 8]; 256],

    crtc: CRTC6845,

    screen_dimensions: (f32, f32),

    color: u8,
}

impl CGA {
    pub fn new(dimensions: (f32, f32)) -> Self {
        let mut file =
            std::fs::File::open("roms/IBM_5788005_AM9264_1981_CGA_MDA_CARD.BIN").unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();

        Self {
            img_buffer: vec![0x00; (dimensions.0 * dimensions.1 * 4.) as usize],
            font_map: decode_font_map(&buf[0x1800..]),
            crtc: CRTC6845::default(),
            screen_dimensions: dimensions,

            color: 0,
        }
    }

    fn enabled(&self) -> bool {
        self.crtc.op1 & 0b00001000 > 0
    }

    fn alphanumeric_mode(&mut self, vram: &[u8]) {
        let screen_character_width = (self.screen_dimensions.0 / 8.) as usize;

        self.img_buffer
            .par_chunks_mut(8 * 4)
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

    fn graphic_mode(&mut self, vram: &[u8]) {
        // TODO GRAPHIC MODE
        // self.img_buffer =
        //     vec![0xFF; (self.screen_dimensions.0 * self.screen_dimensions.1) as usize * 4];
        let palette = (self.color & 0b00100000 != 0) as usize * 2;
        let intensity = (self.color & 0b00010000 != 0) as usize;

        self.img_buffer
            .par_chunks_mut(self.screen_dimensions.0 as usize * 4)
            .enumerate()
            .for_each(|(i, pixel_slice)| {
                let row_slice = if i % 2 == 0 {
                    let slice_start = i * self.screen_dimensions.0 as usize / 8;
                    let slice_end = slice_start + self.screen_dimensions.0 as usize / 4;
                    &vram[slice_start..slice_end]
                } else {
                    let slice_start = (i - 1) * self.screen_dimensions.0 as usize / 8;
                    let slice_end = slice_start + self.screen_dimensions.0 as usize / 4;
                    &vram[(0x2000 + slice_start)..(0x2000 + slice_end)]
                };

                for (group_index, pixel_group) in row_slice.iter().enumerate() {
                    for pixel_offset in 0..4 {
                        let pixel = (pixel_group >> 2 * (3 - pixel_offset)) & 3;
                        let color = PALETTE[palette + intensity][pixel as usize];
                        let color_bytes: [u8; 4] = unsafe { transmute(color.to_be()) };
                        let pixel_slice_start = group_index * 16 + pixel_offset * 4;
                        let pixel_slice_end = pixel_slice_start + 4;

                        pixel_slice[pixel_slice_start..pixel_slice_end].copy_from_slice(&color_bytes)
                    }
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
) -> [u8; 8 * 4] {
    let character_slice = font_map[character.index][row];
    let mut return_slice = [0x00; 8 * 4];

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
            0x3D8 => self.crtc.op1 = val as u8,
            0x3D4 => self.crtc.adddr_reg = (val as u8) as usize,
            // 0x3B5 =>  self.crtc_registers[self.crtc_adddr_reg] = val as u8,
            0x3D5 => self.crtc.reg_write(self.crtc.adddr_reg, val as u8),

            0x3D9 => self.color = val as u8,
            _ => {}
        }
    }
}

impl DisplayAdapter for CGA {
    fn create_frame(&mut self, ctx: &mut ggez::Context, vram: &[u8]) -> Image {
        if !self.enabled() {
            return Image::from_pixels(
                ctx,
                &vec![0x00; (self.screen_dimensions.0 * self.screen_dimensions.1 * 4.) as usize],
                ImageFormat::Rgba8UnormSrgb,
                self.screen_dimensions.0 as u32,
                self.screen_dimensions.1 as u32,
            );
        }

        if !self.get_dirty_vram() {
            // self.add_cursor();
            return Image::from_pixels(
                ctx,
                &self.img_buffer,
                ImageFormat::Rgba8UnormSrgb,
                self.screen_dimensions.0 as u32,
                self.screen_dimensions.1 as u32,
            );
        }

        if self.crtc.op1 & 0b00000010 > 0 {
            self.graphic_mode(vram);
        } else {
            self.alphanumeric_mode(vram);
        }

        self.set_dirty_vram(false);

        Image::from_pixels(
            ctx,
            &self.img_buffer,
            ImageFormat::Rgba8UnormSrgb,
            self.screen_dimensions.0 as u32,
            self.screen_dimensions.1 as u32,
        )
    }

    fn get_dirty_vram(&self) -> bool {
        self.crtc.dirty_vram
    }

    fn set_dirty_vram(&mut self, val: bool) {
        self.crtc.dirty_vram = val
    }

    fn inc_frame_counter(&mut self) {
        self.crtc.frame_counter += 1;
    }
}
