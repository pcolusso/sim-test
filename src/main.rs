use sim_test::{BufferHandle, TwoDeeBuffer};

fn main() {
    let mut buf = TwoDeeBuffer::new(100, 100);
    buf.set(20, 20, 40);


    println!("Hello, world!");
}
