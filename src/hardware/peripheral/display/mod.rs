use eframe::glow::COLOR;
use egui::CollapsingHeader;

pub mod cga;
pub mod crtc6845;
pub mod ibm_mda;
pub mod ega;

pub trait DisplayAdapter {
    fn create_frame(&mut self, vram: &[u8]) -> Vec<u8>;
    fn inc_frame_counter(&mut self);
}

#[derive(Clone, Copy)]
struct Color(u8, u8, u8, u8);

impl Color {
    pub const WHITE: Color = Color(0xFF, 0xFF, 0xFF, 0xFF);
    pub const DARK_GREY: Color = Color(0x66, 0x66, 0x66, 0xFF);
    pub const GREY: Color = Color(0xAA, 0xAA, 0xAA, 0xFF);

    pub const BLACK: Color = Color(0x00, 0x00, 0x00, 0x00);

    pub const GREEN: Color = Color(0x00, 0xC0, 0x00, 0xFF);
    pub const DARK_GREEN: Color = Color(0x00, 0x40, 0x00, 0xFF);
    pub const BRIGHT_GREEN: Color = Color(0x00, 0xFF, 0x00, 0xFF);

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Color {
        Color(r, g, b, 0xFF)
    }
}

trait Character {
    fn decode_colors(self, attr: u8) -> Self;
    fn get_foreground_color(&self) -> &Color;
    fn get_background_color(&self) -> &Color;
}

pub struct BWChar {
    pub index: usize,
    background_color: Color,
    foreground_color: Color,
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
        }
    }
}

impl Character for BWChar {
    fn decode_colors(mut self, attr: u8) -> Self {
        match attr {
            0x00 | 0x08 | 0x80 | 0x88 => {
                self.foreground_color = Color::BLACK;
                self.background_color = Color::BLACK;
            }
            0x70 => {
                self.foreground_color = Color::BLACK;
                self.background_color = Color::GREY;
            }
            0x78 => {
                self.foreground_color = Color::DARK_GREY;
                self.background_color = Color::GREY;
            }
            0xF0 => {
                //TODO BLINK

                self.foreground_color = Color::BLACK;
                self.background_color = Color::WHITE;
            }
            0xF8 => {
                //TODO BLINK

                self.foreground_color = Color::DARK_GREY;
                self.background_color = Color::WHITE;
            }

            _ => {
                let low_nibble = attr & 0x0F;

                self.background_color = Color::BLACK;

                if low_nibble <= 0x07 {
                    self.foreground_color = Color::DARK_GREY;
                } else {
                    self.foreground_color = Color::WHITE;
                }
            }
        }

        self
    }

    fn get_foreground_color(&self) -> &Color {
        &self.foreground_color
    }

    fn get_background_color(&self) -> &Color {
        &self.background_color
    }
}

pub struct ColorChar {
    pub index: usize,
    background_color: Color,
    foreground_color: Color,

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

    fn get_background_color(&self) -> &Color {
        &self.background_color
    }

    fn get_foreground_color(&self) -> &Color {
        &self.foreground_color
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
        .chunks_mut(3)
        .zip(character_slice.iter())
        .for_each(|(pixels, val)| {
            let color = if *val {
                character.get_foreground_color()
            } else {
                character.get_background_color()
            };

            pixels[0] = color.0;
            pixels[1] = color.1;
            pixels[2] = color.2;
            // pixels[3] = color.3;
        });
}
