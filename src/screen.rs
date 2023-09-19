use ggez::glam::Vec2;
use ggez::graphics::{DrawParam, Image, ImageFormat, BlendMode};
use ggez::input::keyboard::KeyInput;
use ggez::{timer, Context};
// A
use crate::hardware::display::DisplayAdapter;
pub use crate::hardware::sys::System;

pub use ggez::conf::WindowMode;
pub use ggez::event::{self, EventHandler};
pub use ggez::graphics::{self, Color};
pub use ggez::input::keyboard::KeyCode;
pub use ggez::{GameError, GameResult};

pub const DESIRED_FPS: f32 = 50.;

pub struct IbmPc {
    pub sys: System,
    img: Image,
    dirty: bool,
}

impl IbmPc {
    // pub fn new(ctx: &Context) -> Self {
    pub fn new(ctx: &Context, sw1: u8, sw2: u8, dimensions: (f32, f32)) -> Self {
        IbmPc {
            sys: System::new(sw1, sw2, dimensions),
            img: Image::from_pixels(
                ctx,
                &vec![0x00; (dimensions.0 * dimensions.1 * 4.) as usize],
                ImageFormat::Rgba8Unorm,
                dimensions.0 as u32,
                dimensions.1 as u32,
            ),
            dirty: false,
        }
    }
}

impl EventHandler for IbmPc {
    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        while ctx.time.check_update_time(DESIRED_FPS as u32) {
            self.sys.update();
            self.sys.bus.mda.inc_frame_counter();
            self.dirty = false;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);
        if !self.dirty {
            self.img = self.sys.create_frame(ctx);
            self.dirty = true;
        }

        canvas.set_sampler(graphics::Sampler::nearest_clamp());
        canvas.draw(&self.img, DrawParam::new().scale(Vec2::new(2., 2.)));
        canvas.finish(ctx)?;

        timer::yield_now();
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        input: KeyInput,
        repeated: bool,
    ) -> Result<(), GameError> {
        if !repeated {
            self.sys.bus.ppi.key_down(input.scancode);
        }

        Ok(())
    }

    fn key_up_event(&mut self, _ctx: &mut Context, input: KeyInput) -> Result<(), GameError> {
        self.sys.bus.ppi.key_up(input.scancode);

        Ok(())
    }
}
