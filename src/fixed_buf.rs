use std::sync::Arc;

use num::{Integer, Num, ToPrimitive};
use crate::buf::{BufferError, TwoDeeBuffer, Flipper};

pub struct FixedTwoDeeBuffer<C: Num + Copy, const W: usize, const H: usize> {
    // It'd be neato if we could have this as a fixed-size array
    // but we can't use those generic values in const expressions.
    // despite the fact, they are very const
    pub buf: Vec<C>,
}

impl<C: Num + Copy, const W: usize, const H: usize>  FixedTwoDeeBuffer<C, W, H> {
    pub fn new(initial: C) -> Self {
        let buf = vec![initial; W * H];
        Self { buf }
    }

    pub const fn size() -> usize {
        size_of::<C>() * W * H
    }


    pub fn row_size(&self) -> usize {
        size_of::<C>() * W
    }

    pub fn height(&self) -> usize {
        H
    }

    pub fn width(&self) -> usize {
        W
    }

    pub const fn len() -> usize {
        W * H
    }
}

impl<C: Num + Copy, const W: usize, const H: usize> TwoDeeBuffer<C> for FixedTwoDeeBuffer<C, W, H> {
    /// Safely retrieves a value from the buffer, for the given x,y coords.
    fn get<T: ToPrimitive>(&self, x: T, y: T) -> Result<C, BufferError> {
        let x = x.to_usize().ok_or(BufferError::BadIndex)?;
        let y = y.to_usize().ok_or(BufferError::BadIndex)?;
        let i = y * self.width() + x;

        self.buf.get(i).copied().ok_or(BufferError::OutOfBounds)
    }

    /// Safely sets a value to the buffer, for the given x,y coords.
    fn set<T: ToPrimitive>(&mut self, x: T, y: T, v: C) -> Result<(), BufferError> {
        let x = x.to_usize().ok_or(BufferError::BadIndex)?;
        let y = y.to_usize().ok_or(BufferError::BadIndex)?;
        let i = y * self.height() + x;

        if i > self.width() * self.height() {
            return Err(BufferError::OutOfBounds);
        }

        self.buf[i] = v;

        Ok(())
    }
}

// We could use a seperate ref that has render, and another that has reader, so that the user
// has a harder time to misues the buffers.
#[derive(Clone)]
pub struct DoubleBuf<const W: usize, const H: usize>(Arc<Flipper<FixedTwoDeeBuffer<u32, W, H>, u32>>);

impl<const W: usize, const H: usize> DoubleBuf<W, H> {
    pub fn new() -> Self {
        let a = FixedTwoDeeBuffer::new(0);
        let b = FixedTwoDeeBuffer::new(0);
        let f = Flipper::new(a, b);
        Self(Arc::new(f))
    }

    // Uses the front buffer, which is safe for read-only access.
    pub fn render<F: Fn(&FixedTwoDeeBuffer<u32, W, H>)>(&self, render_func: F) {
        let f = self.0.front();
        render_func(f);
    }

    // Uses the back buffer, which is not read from.
    pub fn update<F: Fn(&mut FixedTwoDeeBuffer<u32, W, H>)>(&mut self, update_func: F) {
        let x = self.0.clone();
        // SAFETY: Operations only ever occur on the back buffer. Buffers are swapped via
        // an atomic pointer, via flip.
        unsafe {
            let ptr = Arc::into_raw(x) as *mut Flipper<FixedTwoDeeBuffer<u32, W, H>, u32>;
            let buf = (*ptr).back();
            update_func(buf);
            (*ptr).flip();
        }
    }

    pub const fn buf_size() -> usize {
        FixedTwoDeeBuffer::<u8, W, H>::size()
    }
}
