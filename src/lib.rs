pub mod hardware;
pub mod util;

use ggez::Context;
use ggez::glam::Vec2;
use ggez::input::keyboard::KeyInput;
// A
use hardware::display::DisplayAdapter;
pub use hardware::sys::System;

pub use ggez::conf::WindowMode;
pub use ggez::{GameError, GameResult};
pub use ggez::event::{self, EventHandler};
pub use ggez::input::keyboard::KeyCode;
pub use ggez::graphics::{self, Color};

pub const DESIRED_FPS: f32 = 50.;

pub struct IbmPc {
    pub sys: System,
}

impl IbmPc {
    pub fn new() -> Self {
        IbmPc {
            sys: System::new(),
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
        let img = self.sys.bus.mda.create_frame(ctx, &self.sys.bus.memory[0xB0000..0xB0FA0]);

        canvas.draw(&img, Vec2::new(0.0, 0.0));
        canvas.finish(ctx)?;
        Ok(())
    }

    fn key_up_event(&mut self, _ctx: &mut Context, input: KeyInput) -> Result<(), GameError> {
        self.sys.bus.ppi.key_up(input.scancode, &mut self.sys.bus.pic);

        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, repeated: bool) -> Result<(), GameError> {
        if !repeated {
            self.sys.bus.ppi.key_down(input.scancode, &mut self.sys.bus.pic);
        }

        Ok(())
    }
}
