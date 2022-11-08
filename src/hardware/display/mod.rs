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
}

impl Default for Char {
    fn default() -> Self {
        Self {
            index: 0xA5,
            background_color: Color::BLACK,
            foreground_color: Color::WHITE,
        }
    }
}