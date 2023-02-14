use ggez::{Context, graphics::{ImageGeneric, GlBackendSpec, Color}};

pub mod ibm_mda;

pub trait DisplayAdapter {
    fn create_frame(&mut self, ctx: &mut Context, vram: &[u8]) -> ImageGeneric<GlBackendSpec>;
    fn render_font(&mut self, char: Char, width: usize, height: usize);
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
        self.bright = attr & 0x0F > 0x08;
        self.underline = attr & 0x07 == 0x01;

        if matches!(attr, 0x00 | 0x08 | 0x80 | 0x88) {
            self.background_color = Color::BLACK;
            self.foreground_color = Color::BLACK;
        } else if matches!(attr, 0x70 | 0x78 | 0xF0 | 0xF8) {
            self.background_color = Color::WHITE;
            self.foreground_color = Color::BLACK;
        } else {
            self.background_color = Color::BLACK;
            self.foreground_color = Color::WHITE;
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