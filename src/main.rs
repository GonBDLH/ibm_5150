use ibm_5150::*;

// #[cfg(not(debug_assertions))]
fn main() -> GameResult {
    let win_mode = WindowMode::default()
        .dimensions(720., 350.)
        .resize_on_scale_factor_change(true);

    let cb = ggez::ContextBuilder::new("IBM 5150", "Gonzalo").window_mode(win_mode);

    let (ctx, event_loop) = cb.build()?;

    let mut app = IbmPc::new(&ctx);
    //graphics::set_mode(&mut ctx, win_mode)?;

    app.sys.rst();
    app.sys.load_roms();

    event::run(ctx, event_loop, app);
}

// #[cfg(debug_assertions)]
// fn main() {
//     let mut app = IbmPc::new();

//     app.sys.rst();
//     app.sys.load_roms();

//     loop {
//         app.sys.update();
//     }
// }
