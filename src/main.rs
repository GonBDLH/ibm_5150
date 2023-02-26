use ibm_5150::{*, hardware::display::ibm_mda::{IMG_WIDTH, IMG_HEIGHT}};
use notan::{prelude::*, draw::DrawConfig};
use simplelog::*;

// #[cfg(not(debug_assertions))]
// fn main() -> GameResult {
//     let win_mode = WindowMode::default()
//                             .dimensions(720., 350.)
//                             .resize_on_scale_factor_change(true);

//     let cb = ggez::ContextBuilder::new("IBM 5150", "Gonzalo").window_mode(win_mode);
 

//     let (ctx, event_loop) = cb.build()?;

//     let mut app = IbmPc::new(&ctx);
//     //graphics::set_mode(&mut ctx, win_mode)?;

//     app.sys.rst();
//     app.sys.load_roms();

//     event::run(ctx, event_loop, app);
// }

// #[cfg(debug_assertions)]
// fn main() {
//     let mut app = IbmPc::new();

//     app.sys.rst();
//     app.sys.load_roms();

//     loop {
//         app.sys.update();
//     }
// }

#[notan_main]
fn main() -> Result<(), String> {
    SimpleLogger::init(LevelFilter::Info, Config::default()).unwrap();
    
    let win_cfg = WindowConfig::new().size(IMG_WIDTH as i32, IMG_HEIGHT as i32);

    notan::init_with(setup)
        .add_config(win_cfg)
        .add_config(DrawConfig)
        .update(update)
        .draw(draw)
        .build()
}