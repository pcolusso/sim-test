use std::sync::atomic::{AtomicPtr, Ordering};
use num::ToPrimitive;
use rayon::prelude::*;
use thiserror::Error;

const WIDTH: usize = 250;
const HEIGHT: usize = 250;
const BUFFER_SIZE: usize = WIDTH * HEIGHT;

enum BufferSelector {
    A, B
}

struct Buffer {
    width: usize,
    height: usize,
    a: Box<[u8; BUFFER_SIZE]>,
    b: Box<[u8; BUFFER_SIZE]>,
    active: AtomicPtr<u8>
}

#[derive(Error, Debug)]
enum BufferError {
    #[error("Index out of bounds")]
    OutOfBounds,
    #[error("Bad index passed")]
    BadIndex,
}

impl Buffer {
    fn new() -> Self {
        let mut a = Box::new([0; BUFFER_SIZE]);
        let b = Box::new([0; BUFFER_SIZE]);
        let active = AtomicPtr::new(a.as_mut_ptr());
        Self { a, b, width: WIDTH, height: HEIGHT, active }
    }

    // Flip front and back, to be called after every processing step.
    fn flip(&mut self) {
        let current = self.active.load(Ordering::Relaxed);
        if current == self.a.as_mut_ptr() {
            self.active.store(self.b.as_mut_ptr(), Ordering::Release);
        } else {
            self.active.store(self.b.as_mut_ptr(), Ordering::Release);
        }
    }

    // Buffer that is safe to read from. We render from this one.
    fn front(&self) -> &[u8] {
        let ptr = self.active.load(Ordering::Acquire);
        // TODO: Safety?
        unsafe { std::slice::from_raw_parts(ptr, BUFFER_SIZE )}
    }

    // Buffer we operate on.
    fn back(&mut self) -> &mut [u8] {
        let active_ptr = self.active.load(Ordering::Relaxed);
        if active_ptr == self.a.as_mut_ptr() {
            &mut self.b[..]
        } else {
            &mut self.a[..]
        }
    }

    fn get<T: ToPrimitive>(&self, x: T, y: T) -> Result<&u8, BufferError> {
        let x = x.to_usize().ok_or(BufferError::BadIndex)?;
        let y = y.to_usize().ok_or(BufferError::BadIndex)?;
        let i = y * self.width + x;

        self.front().get(i).ok_or(BufferError::OutOfBounds)
    }

    fn set<T: ToPrimitive>(&mut self, x: T, y: T, v: u8) -> Result<(), BufferError> {
        let x = x.to_usize().ok_or(BufferError::BadIndex)?;
        let y = y.to_usize().ok_or(BufferError::BadIndex)?;
        let i = y * self.width + x;

        if i > self.width * self.height {
            return Err(BufferError::OutOfBounds)
        }

        self.back()[i] = v;

        Ok(())
    }
}

fn main() {

    println!("Hello, world!");
}
