use std::io::Read;

use ggez::{graphics::{ImageGeneric, GlBackendSpec, Image}, Context};

use crate::hardware::peripheral::Peripheral;

use super::{DisplayAdapter, Char};

const IMG_BUFF_SIZE: usize = 720 * 350 * 4;
const IMG_SIZE: usize = 720 * 350;

#[derive(Clone)]
pub struct IbmMDA {
    pub img_buffer: Vec<u8>,
    pub font: Vec<u8>,

    crt_op1: u8,
    crt_sp: u8,
}

impl IbmMDA {
    pub fn new() -> IbmMDA {
        // let a: Vec<u8> = (0..IMG_BUFF_SIZE).map(|x| if x % 4 == 3 {0xFF} else {0x00}).collect();
        let a = vec![0x00; IMG_BUFF_SIZE];
        let mut file = std::fs::File::open("roms/IBM_5788005_AM9264_1981_CGA_MDA_CARD.BIN").unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();

        IbmMDA {
            img_buffer: a,
            font: buf,

            crt_op1: 0b00001001,
            crt_sp: 0b00001000,
        }
    }
}

impl DisplayAdapter for IbmMDA {
    fn create_frame(&mut self, ctx: &mut Context, vram: &[u8]) -> ImageGeneric<GlBackendSpec> {
        let mut iter = vram.chunks(2).enumerate();
        
        while let Some(v) = iter.next() {
            let character = Char { index: v.1[0] as usize, ..Default::default() };
            self.render_font(character, v.0 % 80, (v.0 / 80) as usize);
        }

        Image::from_rgba8(ctx, 720, 350, &self.img_buffer).unwrap()
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
                } else {
                    if character.index >= 0xC0 && character.index <= 0xDF {
                        char_ & 1
                    } else {
                        0
                    }
                };
    
                let bg_colors = character.background_color.to_rgba();
                let fg_colors = character.foreground_color.to_rgba();
                // TODO Cambiar prints por correspondiente codigo para poner pixeles en la imagen
                if pixel > 0 {
                    self.img_buffer[((height * 14 + i) * 720 + (width * 9 + j)) * 4 + 0] = fg_colors.0;
                    self.img_buffer[((height * 14 + i) * 720 + (width * 9 + j)) * 4 + 1] = fg_colors.1;
                    self.img_buffer[((height * 14 + i) * 720 + (width * 9 + j)) * 4 + 2] = fg_colors.2;
                    self.img_buffer[((height * 14 + i) * 720 + (width * 9 + j)) * 4 + 3] = fg_colors.3;
                    // self.img_buffer[((height + i) * 720 + (width + j)) * 4 + 0] = fg_colors.0;
                    // self.img_buffer[((height + i) * 720 + (width + j)) * 4 + 1] = fg_colors.1;
                    // self.img_buffer[((height + i) * 720 + (width + j)) * 4 + 2] = fg_colors.2;
                    // self.img_buffer[((height + i) * 720 + (width + j)) * 4 + 3] = fg_colors.3;
                } else {
                    self.img_buffer[((height * 14 + i) * 720 + (width * 9 + j)) * 4 + 0] = bg_colors.0;
                    self.img_buffer[((height * 14 + i) * 720 + (width * 9 + j)) * 4 + 1] = bg_colors.1;
                    self.img_buffer[((height * 14 + i) * 720 + (width * 9 + j)) * 4 + 2] = bg_colors.2;
                    self.img_buffer[((height * 14 + i) * 720 + (width * 9 + j)) * 4 + 3] = bg_colors.3;
                };
                
            }
        }
    }
}

impl Peripheral for IbmMDA {
    fn port_in(&mut self, port: u16) -> u16 {
        match port {
            0x3B8 => self.crt_op1 as u16,
            0x3BA => self.crt_sp as u16,

            _ => 0 //TODO
        }
    }

    fn port_out(&mut self, val: u16, port: u16) {
        match port {
            0x3B8 => self.crt_op1 = val as u8,
            _ => {}   
        }
    }
}