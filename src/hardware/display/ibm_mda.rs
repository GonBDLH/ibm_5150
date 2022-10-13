use std::io::Read;

use ggez::{graphics::{ImageGeneric, GlBackendSpec, Image}, Context};

const IMG_BUFF_SIZE: usize = 720 * 350 * 4;
const IMG_SIZE: usize = 720 * 350;

#[derive(Clone)]
pub struct IbmMDA {
    pub img_buffer: Vec<u8>,
    pub font: Vec<u8>
}

impl IbmMDA {
    pub fn new() -> IbmMDA {
        let a: Vec<u8> = (0..IMG_BUFF_SIZE).map(|x| if x % 4 == 3 {0xFF} else {0x00}).collect();
        let mut file = std::fs::File::open("roms/IBM_5788005_AM9264_1981_CGA_MDA_CARD.BIN").unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();

        IbmMDA {
            img_buffer: a,
            font: buf
        }
    }

    pub fn create_frame(&self, ctx: &mut Context) -> ImageGeneric<GlBackendSpec> {
        Image::from_rgba8(ctx, 720, 350, &self.img_buffer).unwrap()
    }
}
