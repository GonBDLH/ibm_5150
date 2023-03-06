pub mod hardware;
pub mod util;

use ggez::glam::Vec2;
use ggez::graphics::{Image, ImageFormat};
use ggez::input::keyboard::KeyInput;
use ggez::{timer, Context};
// A
use hardware::display::ibm_mda::IMG_BUFF_SIZE;
use hardware::display::DisplayAdapter;
pub use hardware::sys::System;

pub use ggez::conf::WindowMode;
pub use ggez::event::{self, EventHandler};
pub use ggez::graphics::{self, Color};
pub use ggez::input::keyboard::KeyCode;
pub use ggez::{GameError, GameResult};

pub const DESIRED_FPS: f32 = 50.;

pub struct IbmPc {
    pub sys: System,
    img: Image,
}

impl IbmPc {
    pub fn new(ctx: &Context) -> Self {
        IbmPc {
            sys: System::new(),
            img: Image::from_pixels(
                ctx,
                &[0x00; IMG_BUFF_SIZE],
                ImageFormat::Rgba8Unorm,
                720,
                350,
            ),
        }
    }
}

impl EventHandler for IbmPc {
    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        // let mut veces = 0;

        while ctx.time.check_update_time(DESIRED_FPS as u32) {
            self.sys.update();
            // veces += 1;
        }

        // println!("{}", ggez::timer::fps(ctx));

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);
        self.img = self
            .sys
            .bus
            .mda
            .create_frame(ctx, &self.sys.bus.memory[0xB0000..0xB0FA0]);

        canvas.draw(&self.img, Vec2::new(0.0, 0.0));
        canvas.finish(ctx)?;

        timer::yield_now();
        Ok(())
    }

    fn key_up_event(&mut self, _ctx: &mut Context, input: KeyInput) -> Result<(), GameError> {
        self.sys
            .bus
            .ppi
            .key_up(input.scancode, &mut self.sys.bus.pic);

        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        input: KeyInput,
        repeated: bool,
    ) -> Result<(), GameError> {
        if !repeated {
            self.sys
                .bus
                .ppi
                .key_down(input.scancode, &mut self.sys.bus.pic);
        }

        Ok(())
    }
}
