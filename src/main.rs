use ibm_5150::{frontend::{Application, EmulatorConfig}, hardware::{switches_cfg::*, sys::ScreenMode}};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    {
        pollster::block_on(run())
    }
}

async fn run() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = Application::new(EmulatorConfig {
        sw1: DD_ENABLE | RESERVED | MEM_64K | DISPLAY_CGA_40_25 | DRIVES_2,
        sw2: HIGH_NIBBLE | TOTAL_RAM_64,
        screen_mode: ScreenMode::CGA4025,
    });

    event_loop.run_app(&mut app).expect("Failed to run app");
}
