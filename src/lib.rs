mod buf;
mod fixed_buf;
pub use fixed_buf::*;

pub type MyBuf = DoubleBuf<100, 100>;

mod gfx;
pub use gfx::*;
