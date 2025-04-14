use ibm_5150::frontend::{app_state::{bool_array_to_u8, u8_to_bool_array}, Application};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    {
        pollster::block_on(run())
    }
}

async fn run() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = Application::new();

    event_loop.run_app(&mut app).expect("Failed to run app");
}
