use notan::prelude::*;

pub const IMG_BUFF_SIZE: usize = 720 * 350 * 4;

pub mod crtc6845;
pub mod ibm_mda;

pub trait DisplayAdapter {
    fn create_frame(&mut self, gfx: &mut Graphics, texture: &mut Texture, vram: &[u8]);
    fn get_crtc(&self) -> &crtc6845::CRTC6845;
}

pub struct Char {
    pub index: usize,
    pub background_color: Color,
    pub foreground_color: Color,

    pub bright: bool,
    pub underline: bool,
}

impl Char {
    fn new(index: usize) -> Self {
        Char {
            index,
            ..Default::default()
        }
    }

    fn decode_colors(mut self, attr: u8) -> Self {
        self.bright = attr & 0x08 > 0;
        self.underline = attr & 0x07 == 0x01;

        let back = attr >> 4 & 0x07;
        let front = attr & 0x07;

        if attr != 0x07 {
            let _a = 0;
        }

        match (back, front) {
            (0b000, 0b111) => {
                self.foreground_color = Color::WHITE;
                self.background_color = Color::BLACK;
            }
            (0b111, 0b000) => {
                self.foreground_color = Color::BLACK;
                self.background_color = Color::WHITE;
            }
            (0b000, 0b000) => {
                self.foreground_color = Color::BLACK;
                self.background_color = Color::BLACK;
            }
            (0b111, 0b111) => {
                self.foreground_color = Color::WHITE;
                self.background_color = Color::WHITE;
            }

            _ => {
                self.foreground_color = Color::WHITE;
                self.background_color = Color::BLACK;
            }
        }

        self
    }
}

impl Default for Char {
    fn default() -> Self {
        Self {
            index: 0x00,
            background_color: Color::BLACK,
            foreground_color: Color::WHITE,

            bright: false,
            underline: false,
        }
    }
}

trait Enable {
    fn enabled(&self) -> bool;
}

impl<T> Enable for T where T: DisplayAdapter {
    fn enabled(&self) -> bool {
        self.get_crtc().op1 & 0b00001000 > 0
    }
}