pub mod hardware;
pub mod util;

// A
use ggez::graphics::{Drawable, DrawParam};
use hardware::display::DisplayAdapter;
pub use hardware::sys::System;

pub use ggez::conf::WindowMode;
pub use ggez::{GameError, GameResult};
pub use ggez::event::{self, EventHandler};
pub use ggez::input::keyboard::KeyCode;
pub use ggez::graphics::{self, Color};
pub use ggez::timer::check_update_time;

pub const DESIRED_FPS: f32 = 50.;

pub struct IbmPc {
    pub sys: System,
}

impl IbmPc {
    pub fn new() -> Self {
        IbmPc {
            sys: System::new()
        }
    }
}

impl EventHandler for IbmPc {
    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        // let mut veces = 0;

        while check_update_time(ctx, DESIRED_FPS as u32) {
            self.sys.update();
            // veces += 1;
        }

        // println!("{}", ggez::timer::fps(ctx));

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        // graphics::clear(ctx, Color::RED);
        // TODO
        let img = self.sys.bus.mda.create_frame(ctx, &self.sys.bus.memory[0xB0000..0xB0FA0]);

        img.draw(ctx, DrawParam::default())?;

        graphics::present(ctx)
    }

    fn key_up_event(&mut self, _ctx: &mut ggez::Context, keycode: event::KeyCode, _keymods: event::KeyMods) {
        self.sys.bus.ppi.key_up(keycode, &mut self.sys.bus.pic);
    }

    fn key_down_event(&mut self, _ctx: &mut ggez::Context, keycode: event::KeyCode, _keymods: event::KeyMods, repeat: bool,) {
        if !repeat {
            self.sys.bus.ppi.key_down(keycode, &mut self.sys.bus.pic);
        }
    }
}
