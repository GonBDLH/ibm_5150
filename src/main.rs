use ibm_5150::debugger::*;
use ibm_5150::screen::*;
use std::env;

use egui::vec2;

fn main_debugger() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        resizable: false,
        initial_window_size: Some(vec2(1000., 700.)),
        ..Default::default()
    };

    eframe::run_native(
        "Prueba",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

fn main_view() -> GameResult {
    let win_mode = WindowMode::default()
        .dimensions(720., 350.)
        .resize_on_scale_factor_change(true);

    let cb = ggez::ContextBuilder::new("IBM 5150", "Gonzalo").window_mode(win_mode);

    let (ctx, event_loop) = cb.build()?;

    let mut app = IbmPc::new();
    //graphics::set_mode(&mut ctx, win_mode)?;

    app.sys.rst();
    app.sys.load_roms();
    // app.sys
    //     .disk_ctrl
    //     .insert_disk(&mut app.sys.bus, 0, "roms/dos/Disk01.img");
    // app.sys
    //     .disk_ctrl
    //     .insert_disk(&mut app.sys.bus, 1, "roms/dos/Disk02.img");

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
