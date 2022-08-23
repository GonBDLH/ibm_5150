mod hardware;
mod util;

// PRUEBA

use hardware::sys::System;

fn main() {
    let mut sys = System::new();
    
    sys.run();
}
