mod hardware;
mod util;


use hardware::sys::System;

fn main() {
    let mut sys = System::new();

    // dissasemble_all(&sys.bus);
    
    sys.run();
}