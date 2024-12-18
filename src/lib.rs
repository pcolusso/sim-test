mod buf;
mod fixed_buf;
pub use fixed_buf::*;
pub use buf::*;

pub type MyBuf = DoubleBuf<100, 100>;

mod gfx;
pub use gfx::*;

// wtf is abgr? I think we've fucked endianess...
pub fn pack_rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((a as u32) << 24) | ((b as u32) << 16) | ((g as u32) << 8) | (r as u32)
}
