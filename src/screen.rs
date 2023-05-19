pub const DESIRED_FPS: f32 = 50.;

use notan::draw::{CreateDraw, DrawImages};
use notan::prelude::*;
use crate::hardware::sys::System;
use crate::hardware::display::{IMG_BUFF_SIZE, DisplayAdapter};

#[derive(AppState)]
pub struct IbmPc {
    pub sys: System,
    count: f32,
    texture: Texture,
    // key: Option<KeyCode>
}

impl IbmPc {
    pub fn new(gfx: &mut Graphics) -> Self {
        let mut state = Self {
            sys: System::new(),
            count: 0.0,
            texture: gfx.create_texture()
                .from_bytes(&[0x00; IMG_BUFF_SIZE], 720, 350)
                .build()
                .unwrap(),
            // key: None
        };

        state.sys.rst();
        state.sys.load_roms();

        state
    }

}

pub fn update(app: &mut App, state: &mut IbmPc) {
    state.count += app.timer.delta_f32();

    if state.count >= 0.02 {

        state.count = 0.0;

        state.sys.update();
    }
}

pub fn draw(gfx: &mut Graphics, state: &mut IbmPc) {
    state.sys.bus.mda.create_frame(gfx, &mut state.texture, &state.sys.bus.memory[0xB0000..0xB0FA0]);
    let mut draw = gfx.create_draw();
    draw.clear(Color::BLACK);
    draw.image(&state.texture);
    gfx.render(&draw);
}
