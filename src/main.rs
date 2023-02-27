use ibm_5150::*; 

use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 720;
const HEIGHT: usize = 350;

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut sys = IbmPc::new();
    sys.sys.load_roms();

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(20000)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        sys.update();
        sys.get_frame(&mut buffer);

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}