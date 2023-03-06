use std::io::Read;

use ggez::{
    graphics::{Image, ImageFormat},
    Context,
};

use crate::hardware::peripheral::Peripheral;

use super::{crtc6845::CRTC6845, Char, DisplayAdapter};

pub const IMG_BUFF_SIZE: usize = 720 * 350 * 4;
// const IMG_SIZE: usize = 720 * 350;

#[allow(dead_code)]
#[derive(Clone)]
pub struct IbmMDA {
    pub img_buffer: Vec<u8>,
    pub font: Vec<u8>,

    crtc_op1: u8,
    crtc_sp: u8,

    crtc_adddr_reg: usize,
    // crtc_registers: [u8; 18],
    crtc: CRTC6845,

    retrace: u8,
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
            font: buf,

            crtc_op1: 0b00001001,
            crtc_sp: 0b11111110,

            crtc_adddr_reg: 0,
            // crtc_registers: [0x00; 18],
            crtc: CRTC6845::default(),

            retrace: 0,
        }
    }

    fn enabled(&self) -> bool {
        self.crtc_op1 & 0b00001000 > 0
    }
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

        let iter = vram.chunks(2).enumerate();
        for v in iter {
            let character = Char::new(v.1[0] as usize).decode_colors(v.1[1]);

            self.render_font(character, v.0 % 80, v.0 / 80);
        }

        Image::from_pixels(ctx, &self.img_buffer, ImageFormat::Rgba8Unorm, 720, 350)
    }

    fn render_font(&mut self, character: Char, width: usize, height: usize) {
        for i in 0..14 {
            let char_ = if i < 8 {
                self.font[i + character.index * 8]
            } else {
                self.font[0x800 + (i - 8) + character.index * 8]
            };

            for j in 0..9 {
                let pixel = if j < 8 {
                    char_ & (1 << (7 - j))
                } else if character.index >= 0xC0 && character.index <= 0xDF {
                    char_ & 1
                } else {
                    0
                };

                let color = if pixel > 0 {
                    character.foreground_color.to_rgba()
                } else {
                    character.background_color.to_rgba()
                };

                self.img_buffer[((height * 14 + i) * 720 + (width * 9 + j)) * 4] = color.0;
                self.img_buffer[((height * 14 + i) * 720 + (width * 9 + j)) * 4 + 1] = color.1;
                self.img_buffer[((height * 14 + i) * 720 + (width * 9 + j)) * 4 + 2] = color.2;
                self.img_buffer[((height * 14 + i) * 720 + (width * 9 + j)) * 4 + 3] = color.3;
            }
        }
    }
}

impl Peripheral for IbmMDA {
    fn port_in(&mut self, port: u16) -> u16 {
        match port {
            0x3B8 => self.crtc_op1 as u16,
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
            0x3B5 => self.crtc.read_reg(self.crtc_adddr_reg) as u16,

            _ => 0, //TODO
        }
    }

    fn port_out(&mut self, val: u16, port: u16) {
        match port {
            0x3B8 => self.crtc_op1 = val as u8,
            0x3B4 => self.crtc_adddr_reg = (val as u8) as usize,
            // 0x3B5 =>  self.crtc_registers[self.crtc_adddr_reg] = val as u8,
            0x3B5 => self.crtc.reg_write(self.crtc_adddr_reg, val as u8),
            _ => {}
        }
    }
}
