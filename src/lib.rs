pub mod hardware;
pub mod util;

// A
use ggez::graphics::{Drawable, DrawParam};
use hardware::display::DisplayAdapter;
pub use hardware::sys::System;

pub use ggez::conf::WindowMode;
pub use ggez::{GameError, GameResult};
pub use ggez::event::{self, EventHandler};
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

fn decode_key(keycode: event::KeyCode) -> u8 {
    match keycode {
        event::KeyCode::Escape => 1,
        event::KeyCode::Key1 => 2, 
        event::KeyCode::Key2 => 3,
        event::KeyCode::Key3 => 4,
        event::KeyCode::Key4 => 5,
        event::KeyCode::Key5 => 6,
        event::KeyCode::Key6 => 7,
        event::KeyCode::Key7 => 8,
        event::KeyCode::Key8 => 9,
        event::KeyCode::Key9 => 10,
        event::KeyCode::Key0 => 11,
        _ => 0,
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
        let key_code = decode_key(keycode) + 0x80;
        self.sys.bus.key_input(key_code);
    }

    fn key_down_event(&mut self, _ctx: &mut ggez::Context, keycode: event::KeyCode, _keymods: event::KeyMods, repeat: bool,) {
        if !repeat {
            let key_code = decode_key(keycode);
            self.sys.bus.key_input(key_code);
        }
    }
}
