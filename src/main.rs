use ggez::conf::WindowSetup;
use ibm_5150::debugger::*;
use ibm_5150::hardware::switches_cfg::*;
use ibm_5150::screen::*;
use std::env;

use egui::vec2;

fn main_debugger() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        resizable: false,
        initial_window_size: Some(vec2(1000., 700.)),
        ..Default::default()
    };

    eframe::run_native("Prueba", options, Box::new(|_cc| Box::<MyApp>::default()))
}

fn main_view() -> GameResult {
    let sw1 = DD_ENABLE | RESERVED | MEM_64K | DISPLAY_CGA_40_25 | DRIVES_2;
    let sw2 = HIGH_NIBBLE | PLUS_32;

    let dimensions = match (sw1 & 0b00110000) >> 4 {
        0b00 => panic!("Reserved"),
        0b01 => (320., 200.),
        0b10 => (640., 200.),
        0b11 => (720., 350.),
        _ => unreachable!(),
    };

    let win_mode = WindowMode::default()
        .dimensions(dimensions.0 * 2., dimensions.1 * 2.)
        .resize_on_scale_factor_change(true);

    let win_setup = WindowSetup::default()
        .srgb(true);

    let cb = ggez::ContextBuilder::new("IBM 5150", "Gonzalo").window_mode(win_mode).window_setup(win_setup);

    let (ctx, event_loop) = cb.build()?;

    let mut app = IbmPc::new(&ctx, sw1, sw2, dimensions);
    //graphics::set_mode(&mut ctx, win_mode)?;

    app.sys.rst();
    app.sys.load_roms();

    app.sys
        .disk_ctrl
        .insert_disk(&mut app.sys.bus, 0, "roms/dos/3.00/Disk01.img");
    app.sys
        .disk_ctrl
        .insert_disk(&mut app.sys.bus, 1, "roms/dos/3.00/Disk02.img");
    // app.sys
    //     .disk_ctrl
    //     .insert_disk(&mut app.sys.bus, 0, "roms/dos/1.10/DISK01.IMA");

    event::run(ctx, event_loop, app);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    env_logger::init();

    match args.len() {
        1 => main_view().unwrap(),
        2 => match args[1].as_str() {
            "debugger" => main_debugger().unwrap(),
            _ => panic!("Wrong arguments"),
        },
        _ => panic!("Wrong arguments"),
    }
}
