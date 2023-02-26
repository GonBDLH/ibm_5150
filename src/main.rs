use std::time::Duration;

use ibm_5150::*;
use pixels::{Error, SurfaceTexture, Pixels};
use winit_input_helper::WinitInputHelper;
use game_loop::{game_loop, winit::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder}, Time, TimeTrait};


fn main() -> Result<(), Error> {
    // env_logger::init();
    let event_loop = EventLoop::new();
    
    let mut _input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(720., 350.);
        WindowBuilder::new()
            .with_title("IBM 5150 Emulator")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_max_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(720, 350, surface_texture)?
    };
    let mut state = IbmPc::new(pixels);
    state.sys.load_roms();

    game_loop(
        event_loop,
        window,
        state,
        DESIRED_FPS,
        0.1,
        move |g| {
            g.game.update();
        },
        move |g| {
            g.game.draw();
            g.game.pixels.render().unwrap();

            let dt = TIMESTEP.as_secs_f64() - Time::now().sub(&g.current_instant());
            if dt > 0.0 {
                std::thread::sleep(Duration::from_secs_f64(dt));
            }
        },
        move |_g, _event| {

        }
    )
}