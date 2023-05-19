use device_query::Keycode;
use ibm_5150::debugger::*;
use ibm_5150::screen::*;
use ibm_5150::hardware::peripheral::ppi_8255::KEY_QUEUE;
use notan::draw::DrawConfig;
use notan::prelude::*;
use notan_extra::*;
use std::env;
use device_query::{DeviceState, DeviceEvents};

use egui::vec2;

fn main_debugger() -> Result<(), eframe::Error> {
    let mut options = eframe::NativeOptions::default();

    options.resizable = false;
    options.initial_window_size = Some(vec2(1000., 700.));

    eframe::run_native(
        "Prueba",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}

fn main_view() -> Result<(), String> {
    let win_config = WindowConfig::new().size(720, 350).vsync(true);

    let device_state = DeviceState::new();

    let _guard = device_state.on_key_down(|key| {
        let key = decode_keycode(key);
        if key != 0 {
            KEY_QUEUE.lock().unwrap().push_front(key);
        }
    });

    let _guard = device_state.on_key_up(|key| {
        let key = decode_keycode(key);
        if key != 0 {
            KEY_QUEUE.lock().unwrap().push_front(key + 0x80);
        }
    });


    notan::init_with(IbmPc::new)
        .add_config(win_config)
        .add_config(DrawConfig)
        // .add_plugin(FpsLimit::new(50))
        .update(update)
        .draw(draw)
        .build()
}

fn decode_keycode(key: &Keycode) -> u8 {
    match key {
        Keycode::Q => 16,
        Keycode::W => 17,
        Keycode::E => 18,
        Keycode::R => 19,
        Keycode::T => 20,
        Keycode::Y => 21,
        Keycode::U => 22,
        Keycode::I => 23,
        Keycode::O => 24,
        Keycode::P => 25,
        Keycode::A => 30,
        Keycode::S => 31,
        Keycode::D => 32,
        Keycode::F => 33,
        Keycode::G => 34,
        Keycode::H => 35,
        Keycode::J => 36,
        Keycode::K => 37,
        Keycode::L => 38,
        Keycode::Z => 44,
        Keycode::X => 45,
        Keycode::C => 46,
        Keycode::V => 47,
        Keycode::B => 48,
        Keycode::N => 49,
        Keycode::M => 50,
        Keycode::Escape => 1,
        Keycode::Key1 => 2,
        Keycode::Key2 => 3,
        Keycode::Key3 => 4,
        Keycode::Key4 => 5,
        Keycode::Key5 => 6,
        Keycode::Key6 => 7,
        Keycode::Key7 => 8,
        Keycode::Key8 => 9,
        Keycode::Key9 => 10,
        Keycode::Key0 => 11,
        Keycode::Backspace => 14,
        Keycode::LShift => 42,
        Keycode::Enter => 28,
        Keycode::Space => 57,
    
        _ => 0x00,
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => main_view().unwrap(),
        2 => match args[1].as_str() {
            "debugger" => main_debugger().unwrap(),
            _ => panic!("Wrong arguments"),
        },
        _ => panic!("Wrong arguments"),
    }
}
