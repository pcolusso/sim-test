use std::time::Duration;

use sim_test::{pack_rgba, App};
use winit::event_loop::{ControlFlow, EventLoop};

use sim_test::{MyBuf, TwoDeeBuffer};

#[pollster::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let buf = MyBuf::new();

    let mut buf2 = buf.clone();
    std::thread::spawn(move || {
        let mut r: u8 = 0;
        loop {
            r = r.saturating_add(1);
            //println!("r is now {}", r);
            buf2.update(|f| {
                f.set(0, 0, pack_rgba(r, 0, 0, 255)).unwrap();
            });
            std::thread::sleep(Duration::from_millis(60));
        }
    });

    let mut app = App::new(buf);
    event_loop.run_app(&mut app).expect("idk");
}
