use sim_test::App;
use winit::event_loop::{ControlFlow, EventLoop};

use sim_test::MyBuf;

#[pollster::main]
async fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let buf = MyBuf::new();

    let mut app = App::new(buf);
    event_loop.run_app(&mut app).expect("idk");
}
