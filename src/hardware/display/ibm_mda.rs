use std::io::Read;

use ggez::{
    graphics::{Image, ImageFormat},
    Context,
};

use rayon::prelude::*;

use crate::hardware::peripheral::Peripheral;

use super::{crtc6845::CRTC6845, Char, DisplayAdapter, IMG_BUFF_SIZE};

// const IMG_SIZE: usize = 720 * 350;

#[allow(dead_code)]
#[derive(Clone)]
pub struct IbmMDA {
    pub img_buffer: Vec<u8>,
    pub font_rom: Vec<u8>,

    font_map: [[[bool; 9]; 14]; 255], //IGUAL ESTO ESTA MAL NO SE: CARACTER / FILA / COLUMNA

    crtc: CRTC6845,

    retrace: u8,
}

fn decode_font_map(font_rom: &[u8]) -> [[[bool; 9]; 14]; 255] {
    let mut font_map = [[[false; 9]; 14]; 255];
    
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
                } else if character >= 0xC0 && character <= 0xDF {
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
    pub fn new() -> IbmMDA {
        // let a: Vec<u8> = (0..IMG_BUFF_SIZE).map(|x| if x % 4 == 3 {0xFF} else {0x00}).collect();
        let a = vec![0x00; IMG_BUFF_SIZE];
        let mut file =
            std::fs::File::open("roms/IBM_5788005_AM9264_1981_CGA_MDA_CARD.BIN").unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();

        IbmMDA {
            img_buffer: a,
            font_rom: buf.clone(),
            font_map: decode_font_map(&buf),

            crtc: CRTC6845::default(),

            retrace: 0,
        }
    }

    fn enabled(&self) -> bool {
        self.crtc.op1 & 0b00001000 > 0
    }
}

fn decode_pixel_slice(font_map: &[[[bool; 9]; 14]; 255], row: usize, character: Char) -> [u8; 9 * 4] {
    let character_slice = font_map[character.index][row];
    let mut return_slice = [0x00; 9 * 4];

    return_slice.chunks_mut(4).zip(character_slice.iter()).for_each(|(pixels, val)| {
        let color = if *val {
            character.foreground_color.to_rgba()
        } else {
            character.background_color.to_rgba()
        };

        pixels[0] = color.0;
        pixels[1] = color.1;
        pixels[2] = color.2;
        pixels[3] = color.3;
    });

    return_slice
}

impl DisplayAdapter for IbmMDA {
    fn create_frame(&mut self, ctx: &mut Context, vram: &[u8]) -> Image {
        if !self.enabled() {
            return Image::from_pixels(
                ctx,
                &[0x00; IMG_BUFF_SIZE],
                ImageFormat::Rgba8Unorm,
                720,
                350,
            );
        }

        self.img_buffer.par_chunks_mut(9 * 4).enumerate().for_each(|(i, pixel_slice)| {
            let col_index = i % 80;
            let row_index = (i / 80) % 14;
            let row_char_index = i / (80 * 14);

            let char_index = (row_char_index * 80 + col_index) * 2;

            let vram_char = vram[char_index] as usize;
            let vram_attr = vram[char_index + 1];

            let character = Char::new(vram_char).decode_colors(vram_attr);

            let new_pixel_slice = decode_pixel_slice(&self.font_map, row_index, character);
            pixel_slice.copy_from_slice(&new_pixel_slice);
        });

        Image::from_pixels(ctx, &self.img_buffer, ImageFormat::Rgba8Unorm, 720, 350)
    }
}

impl Peripheral for IbmMDA {
    fn port_in(&mut self, port: u16) -> u16 {
        match port {
            0x3B8 => self.crtc.op1 as u16,
            // 0x3BA => self.crtc_sp as u16,
            0x3BA => {
                self.retrace = (self.retrace + 1) % 4;
                match self.retrace {
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
