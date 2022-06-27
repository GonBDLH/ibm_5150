mod hardware;
mod util;


use hardware::sys::System;

fn main() {
    let mut sys = System::new();
    
    sys.run();
}