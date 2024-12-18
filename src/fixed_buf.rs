use num::{Num, ToPrimitive};
use crate::buf::BufferError;

pub struct FixedTwoDeeBuffer<C: Num + Copy, const W: usize, const H: usize> {
    // It'd be neato if we could have this as a fixed-size array
    // but we can't use those generic values in const expressions.
    // despite the fact, they are very const
    buf: Vec<C>,
}

impl<C: Num + Copy, const W: usize, const H: usize> FixedTwoDeeBuffer<C, W, H> {
    fn new(initial: C) -> Self {
        let buf = vec![initial; W * H];
        Self { buf }
    }

    pub fn height() -> usize {
        return H;
    }

    pub fn width() -> usize {
        return W;
    }

    /// Safely retrieves a value from the buffer, for the given x,y coords.
    pub fn get<T: ToPrimitive>(&self, x: T, y: T) -> Result<C, BufferError> {
        let x = x.to_usize().ok_or(BufferError::BadIndex)?;
        let y = y.to_usize().ok_or(BufferError::BadIndex)?;
        let i = y * Self::width() + x;

        self.buf.get(i).copied().ok_or(BufferError::OutOfBounds)
    }

    /// Safely sets a value to the buffer, for the given x,y coords.
    pub fn set<T: ToPrimitive>(&mut self, x: T, y: T, v: C) -> Result<(), BufferError> {
        let x = x.to_usize().ok_or(BufferError::BadIndex)?;
        let y = y.to_usize().ok_or(BufferError::BadIndex)?;
        let i = y * Self::width() + x;

        if i > Self::width() * Self::height() {
            return Err(BufferError::OutOfBounds);
        }

        self.buf[i] = v;

        Ok(())
    }
}
