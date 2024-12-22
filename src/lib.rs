mod buf;
mod fixed_buf;
pub use buf::*;
pub use fixed_buf::*;

pub type MyBuf = DoubleBuf<50, 50>;

mod gfx;
pub use gfx::*;

// wtf is abgr? I think we've fucked endianess...
pub fn pack_rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((a as u32) << 24) | ((b as u32) << 16) | ((g as u32) << 8) | (r as u32)
}

pub fn mak_coolor(r: u8, g: u8, b: u8) -> u32 {
    let r5 = ((r as u32) >> 3) & 0x1F;
    let g5 = ((g as u32) >> 3) & 0x1F;
    let b5 = ((b as u32) >> 3) & 0x1F;

    // BGRA5551 format: BBBBBGGGGGRRRRRA
    (b5 << 11) | (g5 << 6) | (r5 << 1) | 1
}
