pub mod hardware;
pub mod util;

// A
use hardware::display::DisplayAdapter;
pub use hardware::sys::System;

pub const DESIRED_FPS: f32 = 50.;

pub struct IbmPc {
    pub sys: System,
}

impl IbmPc {
    pub fn new() -> Self {
        IbmPc {
            sys: System::new()
        }
    }

    pub fn update(&mut self) {
        self.sys.update();
    }

    pub fn get_frame(&mut self, frame: &mut [u32]) {
        self.sys.bus.mda.create_frame(&self.sys.bus.memory[0xB0000..0xB0FA0], frame);
    }
}
