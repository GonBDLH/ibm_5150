use ibm_5150::{frontend::{Application, app_state::EmulatorConfig}, hardware::{switches_cfg::*, sys::ScreenMode}};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    {
        pollster::block_on(run())
    }
}

async fn run() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = Application::new(EmulatorConfig::builder()
        .enable_disk_drives()
        .set_conventional_ram(MEM_64K)
        .set_screen_mode(ScreenMode::CGA4025)
        .set_disk_drives_number(DRIVES_2)
        .set_total_ram(TOTAL_RAM_128)
    );

    event_loop.run_app(&mut app).expect("Failed to run app");
}
