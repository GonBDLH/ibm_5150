use ggez::glam::Vec2;
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
    // img: Image,
}

impl IbmPc {
    // pub fn new(ctx: &Context) -> Self {
    pub fn new() -> Self {
        IbmPc {
            sys: System::new(),
            // img: Image::from_pixels(
            //     ctx,
            //     &[0x00; IMG_BUFF_SIZE],
            //     ImageFormat::Rgba8Unorm,
            //     720,
            //     350,
            // ),
        }
    }
}

impl Default for IbmPc {
    fn default() -> Self {
        IbmPc::new()
    }
}

impl EventHandler for IbmPc {
    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        // let mut veces = 0;

        while ctx.time.check_update_time(DESIRED_FPS as u32) {
            self.sys.update();
            self.sys.bus.mda.frame_counter += 1;
            // veces += 1;
        }

        // println!("{}", ggez::timer::fps(ctx));

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);
            let img = self
                .sys
                .bus
                .mda
                .create_frame(ctx, &self.sys.bus.memory[0xB0000..0xB0FA0]);
            // .create_frame(ctx, &self.sys.bus.memory[0xB0000..0xB1000]);
    
            canvas.draw(&img, Vec2::new(0.0, 0.0));
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
