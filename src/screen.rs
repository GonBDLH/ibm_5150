pub const DESIRED_FPS: f32 = 50.;

use crate::hardware::display::{DisplayAdapter, IMG_BUFF_SIZE};
use crate::hardware::sys::System;
use lazy_static::lazy_static;
use notan::draw::{CreateDraw, DrawImages};
use notan::prelude::*;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    static ref KBD_REPEAT: Mutex<HashMap<KeyCode, bool>> = Mutex::new(HashMap::new());
}

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
            texture: gfx
                .create_texture()
                .from_bytes(&[0x00; IMG_BUFF_SIZE], 720, 350)
                .build()
                .unwrap(),
            // key: None
        };

        state.sys.rst();
        state.sys.load_roms();

        // state.sys
        //     .disk_ctrl
        //     .insert_disk(&mut state.sys.bus, 0, "roms/dos/Disk01.img");
        // state.sys
        //     .disk_ctrl
        //     .insert_disk(&mut state.sys.bus, 1, "roms/dos/Disk02.img");
        // state.sys
        //     .disk_ctrl
        //     .insert_disk(&mut state.sys.bus, 0, "roms/dos/pc-dos-3.20.img");

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
    state.sys.bus.mda.create_frame(
        gfx,
        &mut state.texture,
        &state.sys.bus.memory[0xB0000..0xB0FA0],
    );
    let mut draw = gfx.create_draw();
    draw.clear(Color::BLACK);
    draw.image(&state.texture);
    gfx.render(&draw);
}

pub fn event(state: &mut IbmPc, event: Event) {
    match event {
        Event::KeyDown { key } => {
            let mut kbd_repeat = KBD_REPEAT.lock().unwrap();
            let repeated = kbd_repeat.entry(key).or_insert(false);

            if !*repeated {
                if let Some(scancode) = decode_keycode(&key) {
                    state.sys.bus.ppi.key_down(scancode);
                    kbd_repeat.entry(key).and_modify(|e| *e = true);
                }
            }
        }
        Event::KeyUp { key } => {
            let mut kbd_repear = KBD_REPEAT.lock().unwrap();
            
            if let Some(scancode) = decode_keycode(&key) {
                kbd_repear
                    .entry(key)
                    .and_modify(|e| *e = false)
                    .or_insert(false);
                state.sys.bus.ppi.key_up(scancode);
            }
        }
        _ => {}
    }
}

fn decode_keycode(key: &KeyCode) -> Option<u8> {
    let scancode = match key {
        KeyCode::Q => 16,
        KeyCode::W => 17,
        KeyCode::E => 18,
        KeyCode::R => 19,
        KeyCode::T => 20,
        KeyCode::Y => 21,
        KeyCode::U => 22,
        KeyCode::I => 23,
        KeyCode::O => 24,
        KeyCode::P => 25,
        KeyCode::A => 30,
        KeyCode::S => 31,
        KeyCode::D => 32,
        KeyCode::F => 33,
        KeyCode::G => 34,
        KeyCode::H => 35,
        KeyCode::J => 36,
        KeyCode::K => 37,
        KeyCode::L => 38,
        KeyCode::Z => 44,
        KeyCode::X => 45,
        KeyCode::C => 46,
        KeyCode::V => 47,
        KeyCode::B => 48,
        KeyCode::N => 49,
        KeyCode::M => 50,
        KeyCode::Escape => 1,
        KeyCode::Key1 => 2,
        KeyCode::Key2 => 3,
        KeyCode::Key3 => 4,
        KeyCode::Key4 => 5,
        KeyCode::Key5 => 6,
        KeyCode::Key6 => 7,
        KeyCode::Key7 => 8,
        KeyCode::Key8 => 9,
        KeyCode::Key9 => 10,
        KeyCode::Key0 => 11,
        KeyCode::Apostrophe => 12,
        KeyCode::Back => 14,
        KeyCode::Tab => 15,
        KeyCode::LShift => 42,
        KeyCode::Return => 28,
        KeyCode::Space => 57,
        KeyCode::Capital => 58,
        KeyCode::LControl => 29,
        KeyCode::LAlt => 56,
        KeyCode::Delete => 83,
        KeyCode::Comma => 51,
        KeyCode::Period => 52,
        KeyCode::Minus => 53,

        _ => {
            log::warn!("{:?}", key);
            return None
        },
    };

    Some(scancode)
}
