use ibm_5150::*; 

fn main_rls() -> GameResult {
    let mut app = IbmPc::new();
    let win_mode = WindowMode::default()
                            .dimensions(720., 350.)
                            .resize_on_scale_factor_change(true);

    let cb = ggez::ContextBuilder::new("IBM 5150", "Gonzalo").window_mode(win_mode);
 

    let (ctx, event_loop) = cb.build()?;

    //graphics::set_mode(&mut ctx, win_mode)?;

    app.sys.rst();
    app.sys.load_bios();

    event::run(ctx, event_loop, app);
}

fn main_dbg() {
    let mut app = IbmPc::new();

    app.sys.rst();
    app.sys.load_bios();

    loop {
        app.sys.update();
    }
}

fn main() {
    #[cfg(debug_assertions)]
    main_dbg();

    #[cfg(not(debug_assertions))]
    main_rls();
}