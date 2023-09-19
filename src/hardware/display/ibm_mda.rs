use std::io::Read;

use ggez::{
    graphics::{Image, ImageFormat},
    Context,
};

use crate::hardware::display::process_pixel_slice;
use rayon::prelude::*;

use crate::hardware::peripheral::Peripheral;

use super::{
    crtc6845::{BlinkMode, CRTC6845},
    BWChar, Character, DisplayAdapter,
};

// const IMG_SIZE: usize = 720 * 350;

#[allow(dead_code)]
#[derive(Clone)]
pub struct IbmMDA {
    pub img_buffer: Vec<u8>,
    pub font_rom: Vec<u8>,

    font_map: [[[bool; 9]; 14]; 256],

    crtc: CRTC6845,

    cursor_color: u8,

    screen_dimensions: (f32, f32),
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
    pub fn new(dimensions: (f32, f32)) -> IbmMDA {
        // let a: Vec<u8> = (0..IMG_BUFF_SIZE).map(|x| if x % 4 == 3 {0xFF} else {0x00}).collect();
        let a = vec![0x00; (dimensions.0 * dimensions.1 * 4.) as usize];
        let mut file =
            std::fs::File::open("roms/IBM_5788005_AM9264_1981_CGA_MDA_CARD.BIN").unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();

        IbmMDA {
            img_buffer: a,
            font_rom: buf.clone(),
            font_map: decode_font_map(&buf),

            crtc: CRTC6845::default(),

            cursor_color: 0xFF,

            screen_dimensions: dimensions,
        }
    }

    fn enabled(&self) -> bool {
        self.crtc.op1 & 0b00001000 > 0
    }

    fn add_cursor(&mut self) {
        let (x, y) = self.crtc.get_cursor_xy();
        let cursor_size = self.crtc.get_cursor_start_end();
        let blink_mode = self.crtc.get_cursor_blink();

        for z in 0..14 {
            for t in 0..9 {
                if z < cursor_size.0 || z > cursor_size.1 {
                    continue;
                }

                match blink_mode {
                    BlinkMode::NonBlink => self.cursor_color = 0xFF,
                    BlinkMode::NonDisplay => continue,
                    BlinkMode::Blink1_16 => {
                        if self.crtc.frame_counter % 16 == 0 {
                            self.cursor_color = !self.cursor_color;
                            self.crtc.frame_counter = 1;
                        }
                    }
                    BlinkMode::Blink1_32 => {
                        if self.crtc.frame_counter % 32 == 0 {
                            self.cursor_color = !self.cursor_color;
                            self.crtc.frame_counter = 1;
                        }
                    }
                };

                let index = t + x * 9 + z * 9 * 80 + y * 9 * 80 * 14;
                for j in 0..3 {
                    self.img_buffer[index * 4 + j] = self.cursor_color;
                }
                self.img_buffer[index * 4 + 3] = 0xFF;
            }
        }
    }
}

impl Default for IbmMDA {
    fn default() -> Self {
        IbmMDA::new((720., 350.))
    }
}

fn decode_pixel_slice(
    font_map: &[[[bool; 9]; 14]; 256],
    row: usize,
    character: BWChar,
) -> [u8; 9 * 4] {
    let character_slice = font_map[character.index][row];
    let mut return_slice = [0x00; 9 * 4];

    process_pixel_slice(&mut return_slice, &character_slice, character);

    return_slice
}

impl DisplayAdapter for IbmMDA {
    fn create_frame(&mut self, ctx: &mut Context, vram: &[u8]) -> Image {
        if !self.enabled() {
            return Image::from_pixels(
                ctx,
                &vec![0x00; (self.screen_dimensions.0 * self.screen_dimensions.1 * 4.) as usize],
                ImageFormat::Rgba8Unorm,
                720,
                350,
            );
        }

        if !self.get_dirty_vram() {
            self.add_cursor();
            return Image::from_pixels(ctx, &self.img_buffer, ImageFormat::Rgba8Unorm, 720, 350);
        }

        self.img_buffer
            .par_chunks_mut(9 * 4)
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

        self.add_cursor();
        self.set_dirty_vram(false);

        Image::from_pixels(ctx, &self.img_buffer, ImageFormat::Rgba8Uint, 720, 350)
    }

    fn get_dirty_vram(&self) -> bool {
        self.crtc.dirty_vram
    }

    fn set_dirty_vram(&mut self, val: bool) {
        self.crtc.dirty_vram = val;
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
}
