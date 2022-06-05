mod hardware;

use hardware::sys::System;

fn main() {
    let mut sys = System::new();

    sys.clock();

}
