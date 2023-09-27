use egui::CollapsingHeader;
use ggez::{
    graphics::{Color, Image},
    Context,
};

pub mod cga;
pub mod crtc6845;
pub mod ibm_mda;

pub trait DisplayAdapter {
    fn create_frame(&mut self, ctx: &mut Context, vram: &[u8]) -> Image;
    fn inc_frame_counter(&mut self);
}

trait Character {
    fn decode_colors(self, attr: u8) -> Self;
    fn get_foreground_color(&self) -> Color;
    fn get_background_color(&self) -> Color;
}

pub struct BWChar {
    pub index: usize,
    pub background_color: Color,
    pub foreground_color: Color,

    pub bright: bool,
    pub underline: bool,
}

impl BWChar {
    fn new(index: usize) -> Self {
        BWChar {
            index,
            ..Default::default()
        }
    }
}

impl Default for BWChar {
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

impl Character for BWChar {
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

    fn get_foreground_color(&self) -> Color {
        self.foreground_color
    }

    fn get_background_color(&self) -> Color {
        self.background_color
    }
}

pub struct ColorChar {
    pub index: usize,
    pub background_color: Color,
    pub foreground_color: Color,

    pub bright: bool,
}

impl ColorChar {
    fn new(index: usize) -> Self {
        Self {
            index,
            ..Default::default()
        }
    }

    fn decode_rgb(val: u8) -> Color {
        let intensity = (val & 0b1000 != 0) as u8;
        let red = (val & 0b0100 != 0) as u8;
        let green = (val & 0b0010 != 0) as u8;
        let blue = (val & 0b0001 != 0) as u8;

        Color::from_rgb(
            red * 0xAA + intensity * 0x55,
            green * 0xAA + intensity * 0x55,
            blue * 0xAA + intensity * 0x55,
        )
    }
}

impl Character for ColorChar {
    fn decode_colors(mut self, attr: u8) -> Self {
        self.bright = attr & 0x08 > 0;

        let back = (attr >> 4) & 0x07;
        let front = attr & 0xF;

        if attr != 0x07 {
            let _a = 0;
        }

        self.foreground_color = Self::decode_rgb(front);
        self.background_color = Self::decode_rgb(back);

        self
    }

    fn get_background_color(&self) -> Color {
        self.background_color
    }

    fn get_foreground_color(&self) -> Color {
        self.foreground_color
    }
}

impl Default for ColorChar {
    fn default() -> Self {
        Self {
            index: 0x00,
            background_color: Color::BLACK,
            foreground_color: Color::WHITE,

            bright: false,
        }
    }
}

fn process_pixel_slice(
    return_slice: &mut [u8],
    character_slice: &[bool],
    character: impl Character,
) {
    return_slice
        .chunks_mut(4)
        .zip(character_slice.iter())
        .for_each(|(pixels, val)| {
            let color = if *val {
                character.get_foreground_color().to_rgba()
            } else {
                character.get_background_color().to_rgba()
            };

            pixels[0] = color.0;
            pixels[1] = color.1;
            pixels[2] = color.2;
            pixels[3] = color.3;
        });
}
