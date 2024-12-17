use num::ToPrimitive;
use std::sync::atomic::{AtomicPtr, Ordering};
use thiserror::Error;

pub struct Buffer {
    width: usize,
    height: usize,
    a: Box<Vec<u8>>,
    b: Box<Vec<u8>>,
    active: AtomicPtr<u8>,
}

#[derive(Error, Debug)]
pub enum BufferError {
    #[error("Index out of bounds")]
    OutOfBounds,
    #[error("Bad index passed")]
    BadIndex,
}

impl Buffer {
    pub fn new(width: usize, height: usize) -> Self {
        let mut a = Box::new(Vec::with_capacity(width * height));
        let b = Box::new(Vec::with_capacity(width * height));
        let active = AtomicPtr::new(a.as_mut_ptr());
        Self {
            a,
            b,
            width,
            height,
            active,
        }
    }

    // Flip front and back, to be called after every processing step.
    pub fn flip(&mut self) {
        let current = self.active.load(Ordering::Relaxed);
        if current == self.a.as_mut_ptr() {
            self.active.store(self.b.as_mut_ptr(), Ordering::Release);
        } else {
            self.active.store(self.b.as_mut_ptr(), Ordering::Release);
        }
    }

    pub fn size(&self) -> usize {
        self.width * self.height
    }

    // Buffer that is safe to read from. We render from this one.
    pub fn front(&self) -> &[u8] {
        let ptr = self.active.load(Ordering::Acquire);
        // TODO: Safety?
        unsafe { std::slice::from_raw_parts(ptr, self.size()) }
    }

    // Buffer we operate on.
    pub fn back(&mut self) -> &mut [u8] {
        let active_ptr = self.active.load(Ordering::Relaxed);
        if active_ptr == self.a.as_mut_ptr() {
            &mut self.b[..]
        } else {
            &mut self.a[..]
        }
    }

    pub fn get<T: ToPrimitive>(&self, x: T, y: T) -> Result<&u8, BufferError> {
        let x = x.to_usize().ok_or(BufferError::BadIndex)?;
        let y = y.to_usize().ok_or(BufferError::BadIndex)?;
        let i = y * self.width + x;

        self.front().get(i).ok_or(BufferError::OutOfBounds)
    }

    pub fn set<T: ToPrimitive>(&mut self, x: T, y: T, v: u8) -> Result<(), BufferError> {
        let x = x.to_usize().ok_or(BufferError::BadIndex)?;
        let y = y.to_usize().ok_or(BufferError::BadIndex)?;
        let i = y * self.width + x;

        if i > self.width * self.height {
            return Err(BufferError::OutOfBounds);
        }

        self.back()[i] = v;

        Ok(())
    }

    pub fn render<F: Fn(&[u8])>(&self, render_func: F) {
        let f = self.front();
        render_func(f);
    }

    pub fn update<F: Fn(&mut [u8])>(&mut self, update_func: F) {
        let b = self.back();
        update_func(b);
        self.flip();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_buffer_flip() {
        let mut buffer = Buffer::new(5, 5);
        let initial_active = buffer.active.load(Ordering::Relaxed);
        buffer.flip();
        let flipped_active = buffer.active.load(Ordering::Relaxed);
        assert_ne!(initial_active, flipped_active);
    }

    #[test]
    fn test_buffer_front_and_back() {
        let mut buffer = Buffer::new(3, 3);

        // Set a value in the back buffer
        buffer.back()[0] = 42;

        // Front buffer should be empty
        assert_eq!(buffer.front()[0], 0);

        // Flip the buffers
        buffer.flip();

        // Now the front buffer should have the value
        assert_eq!(buffer.front()[0], 42);
    }

    #[test]
    fn test_buffer_get() {
        let mut buffer = Buffer::new(3, 3);
        buffer.back()[4] = 123;
        buffer.flip();

        let one_one = buffer.get(1, 1).unwrap().clone();
        let zero_zero = buffer.get(0, 0).unwrap().clone();

        assert_eq!(one_one, 123);
        assert_eq!(zero_zero, 0);
        assert!(buffer.get(3, 3).is_err());
    }

    #[test]
    fn test_buffer_set() {
        let mut buffer = Buffer::new(3, 3);

        assert!(buffer.set(1, 1, 42).is_ok());
        buffer.flip();
        assert_eq!(buffer.get(1, 1).unwrap(), &42);

        assert!(buffer.set(3, 3, 0).is_err());
    }

    #[test]
    fn test_buffer_out_of_bounds() {
        let buffer = Buffer::new(5, 5);
        assert!(matches!(buffer.get(5, 5), Err(BufferError::OutOfBounds)));
    }

    #[test]
    fn test_buffer_bad_index() {
        let buffer = Buffer::new(5, 5);
        assert!(matches!(buffer.get(-1, 0), Err(BufferError::BadIndex)));
    }

    #[test]
    fn can_share() {
        let buf = Arc::new(Buffer::new(100, 100));

        let r_buf = buf.clone();
        let renderer = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            for _ in 0..1000 {
                let i = rng.gen_range(0..1000);
                r_buf.render(|f| {
                    let c = f[i];
                    assert_ne!(c, 0);
                    // Assume we render using some UI code here.
                });

                thread::sleep(Duration::from_millis(300))
            }
        });
        let mut w_buf = buf.clone();
        let worker = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            for _ in 0..1000 {
                let i = rng.gen_range(0..1000);
                let c: u8 = rng.gen();
                w_buf.update(|f| {
                    f[i] = c;
                });
                thread::sleep(Duration::from_millis(300));
            }
        });

        worker.join().unwrap();
        renderer.join().unwrap();
    }
}
