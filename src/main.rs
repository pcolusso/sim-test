use sim_test::App;
use winit::event_loop::{ControlFlow, EventLoop};

#[pollster::main]
async fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app).expect("idk");
}
