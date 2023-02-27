pub mod hardware;
pub mod util;

use hardware::display::DisplayAdapter;
use hardware::display::ibm_mda::IMG_BUFF_SIZE;
pub use hardware::sys::System;
use notan::log::info;
use notan::{AppState, Event};
use notan::draw::{CreateDraw, DrawImages};
use notan::prelude::{Graphics, App, Texture};

pub const DESIRED_FPS: f32 = 50.;

#[derive(AppState)]
pub struct IbmPc {
    pub sys: System,
    
    count: f32,
    texture: Texture
}

impl IbmPc {
    fn new(gfx: &mut Graphics) -> Self {
        IbmPc {
            sys: System::new(),
            
            count: 0.0,
            texture: gfx
                .create_texture()
                .from_bytes(&[0x00; IMG_BUFF_SIZE], 720, 350)
                .build()
                .unwrap()
        }
    }
}

pub fn setup(gfx: &mut Graphics) -> IbmPc {
    let mut ibm_pc = IbmPc::new(gfx);
    ibm_pc.sys.load_roms();
    ibm_pc
}

pub fn update(app: &mut App, ibm_pc: &mut IbmPc) {
    ibm_pc.count += app.timer.delta_f32();

    if ibm_pc.count >= 1. / DESIRED_FPS {
        ibm_pc.count = 0.;
        ibm_pc.sys.update();
    }
}

pub fn draw(gfx: &mut Graphics, ibm_pc: &mut IbmPc) {
    ibm_pc.sys.bus.mda.create_frame(gfx, &mut ibm_pc.texture, &ibm_pc.sys.bus.memory[0xB0000..0xB0FA0]);

    let mut draw = gfx.create_draw();
    draw.image(&ibm_pc.texture);
    gfx.render(&draw);
}

pub fn event(_ibm_pc: &mut IbmPc, event: Event) {
    match event {
        Event::KeyDown{key} => {
            // ibm_pc.sys.bus.ppi.key_down
            info!("{:?}", key);
        },
        _ => {},
    }
}

// impl EventHandler for IbmPc {
//     fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
//         // let mut veces = 0;

//         while ctx.time.check_update_time(DESIRED_FPS as u32) {
//             self.sys.update();
//             // veces += 1;
//         }

//         // println!("{}", ggez::timer::fps(ctx));

//         Ok(())
//     }

//     fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
//         let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);
//         self.img = self.sys.bus.mda.create_frame(ctx, &self.sys.bus.memory[0xB0000..0xB0FA0]);

//         canvas.draw(&self.img, Vec2::new(0.0, 0.0));
//         canvas.finish(ctx)?;
        

//         timer::yield_now();
//         Ok(())
//     }

//     fn key_up_event(&mut self, _ctx: &mut Context, input: KeyInput) -> Result<(), GameError> {
//         self.sys.bus.ppi.key_up(input.scancode, &mut self.sys.bus.pic);

//         Ok(())
//     }

//     fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, repeated: bool) -> Result<(), GameError> {
//         if !repeated {
//             self.sys.bus.ppi.key_down(input.scancode, &mut self.sys.bus.pic);
//         }

//         Ok(())
//     }
// }
