use num::{Num, ToPrimitive};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;
use std::usize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BufferError {
    #[error("Index out of bounds")]
    OutOfBounds,
    #[error("Bad index passed")]
    BadIndex,
}

/// Provides simple x,y indexing into a buffer.
pub struct SimpleTwoDeeBuffer {
    width: usize,
    height: usize,
    buf: Vec<u8>,
}

impl SimpleTwoDeeBuffer {
    fn new(width: usize, height: usize) -> Self {
        let buf = vec![0u8; width * height];
        Self { width, height, buf }
    }
}

pub trait TwoDeeBuffer<B: Num + Copy> {
    fn get<T: ToPrimitive>(&self, x: T, y: T) -> Result<B, BufferError>;
    fn set<T: ToPrimitive>(&mut self, x: T, y: T, v: B) -> Result<(), BufferError>;
}

impl TwoDeeBuffer<u8> for SimpleTwoDeeBuffer {
    /// Safely retrieves a value from the buffer, for the given x,y coords.
    fn get<T: ToPrimitive>(&self, x: T, y: T) -> Result<u8, BufferError> {
        let x = x.to_usize().ok_or(BufferError::BadIndex)?;
        let y = y.to_usize().ok_or(BufferError::BadIndex)?;
        let i = y * self.width + x;

        self.buf.get(i).copied().ok_or(BufferError::OutOfBounds)
    }

    /// Safely sets a value to the buffer, for the given x,y coords.
    fn set<T: ToPrimitive>(&mut self, x: T, y: T, v: u8) -> Result<(), BufferError> {
        let x = x.to_usize().ok_or(BufferError::BadIndex)?;
        let y = y.to_usize().ok_or(BufferError::BadIndex)?;
        let i = y * self.width + x;

        if i > self.width * self.height {
            return Err(BufferError::OutOfBounds);
        }

        self.buf[i] = v;

        Ok(())
    }
}

/// Double buffer implementation.
pub struct Flipper<F, T>
where
    T: Num + Copy,
    F: TwoDeeBuffer<T>
{
    a: Box<F>,
    b: Box<F>,
    active: AtomicPtr<F>,
    _marker: PhantomData<T>
}

impl<F, T> Flipper<F, T>
where
    T: Num + Copy,
    F: TwoDeeBuffer<T>
{
    /// Internally creates 2x TwoDeeBuffers.
    pub fn new(a: F, b: F) -> Self {
        let mut a = Box::new(a);
        let b = Box::new(b);
        let active = AtomicPtr::new(a.as_mut());
        let _marker = PhantomData::default();

        Self { a, b, active, _marker }
    }

    /// Flip front and back, to be called after every processing step.
    /// Not to be called directly, but in the wrapper one level up.
    pub fn flip(&mut self) {
        let current = self.active.load(Ordering::Relaxed);
        if current == self.a.as_mut() {
            self.active.store(self.b.as_mut(), Ordering::Release);
        } else {
            self.active.store(self.a.as_mut(), Ordering::Release);
        }
    }

    /// Buffer that is safe to read from. Use this for rendering.
    pub fn front(&self) -> &F {
        let ptr = self.active.load(Ordering::Acquire);
        println!("front ptr is {:p}", ptr);
        // SAFETY: Enforced by the wrapper one level up, after every mutation the buffer is flipped,
        // therefor we can assure nothing is writing to this, and thus is safe to read from.
        unsafe { &*ptr }
    }

    /// Buffer we process on. Used for updates.
    pub fn back(&mut self) -> &mut F {
        let active_ptr = self.active.load(Ordering::Relaxed);
        println!("back ptr is {:p}", active_ptr);
        if active_ptr == self.a.as_mut() {
            &mut self.b
        } else {
            &mut self.a
        }
    }
}

/// Thread-safe handle to the double buffer
#[derive(Clone)]
pub struct BufferHandle(Arc<Flipper<SimpleTwoDeeBuffer, u8>>);

impl BufferHandle {
    pub fn new(width: usize, height: usize) -> Self {
        let a = SimpleTwoDeeBuffer::new(width, height);
        let b = SimpleTwoDeeBuffer::new(width, height);
        Self(Arc::new(Flipper::new(a, b)))
    }

    // Uses the front buffer, which is safe for read-only access.
    pub fn render<F: Fn(&SimpleTwoDeeBuffer)>(&self, render_func: F) {
        let f = self.0.front();
        render_func(f);
    }

    // Uses the back buffer, which is not read from.
    pub fn update<F: Fn(&mut SimpleTwoDeeBuffer)>(&mut self, update_func: F) {
        let x = self.0.clone();
        // SAFETY: Operations only ever occur on the back buffer. Buffers are swapped via
        // an atomic pointer, via flip.
        unsafe {
            let ptr = Arc::into_raw(x) as *mut Flipper<SimpleTwoDeeBuffer, u8>;
            let buf = (*ptr).back();
            update_func(buf);
            (*ptr).flip();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;
    use std::thread;

    #[test]
    fn test_buffer_out_of_bounds() {
        let buffer = SimpleTwoDeeBuffer::new(5, 5);
        assert!(matches!(buffer.get(5, 5), Err(BufferError::OutOfBounds)));
    }

    #[test]
    fn test_buffer_bad_index() {
        let buffer = SimpleTwoDeeBuffer::new(5, 5);
        assert!(matches!(buffer.get(-1, 0), Err(BufferError::BadIndex)));
    }

    #[test]
    fn can_share() {
        let buf = BufferHandle::new(100, 100);

        let r_buf = buf.clone();
        let renderer = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            for _ in 0..1000 {
                let x = rng.gen_range(0..100);
                let y = rng.gen_range(0..100);
                r_buf.render(|f| {
                    let c = f.get(x, y);
                    // Assume we render using some UI code here.
                    println!("Picked {:?}", c);
                });
            }
        });
        let mut w_buf = buf.clone();
        let worker = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            for _ in 0..1000 {
                let x = rng.gen_range(0..100);
                let y = rng.gen_range(0..100);
                let c: u8 = rng.gen();
                w_buf.update(|f| {
                    f.set(x, y, c).unwrap();
                });
            }
        });

        worker.join().unwrap();
        renderer.join().unwrap();
    }

    #[test]
    fn test_concurrent_access() {
        let buf = BufferHandle::new(100, 100);

        // Spin up multiple render threads
        let mut renderers = vec![];
        for _ in 0..5 {
            let r_buf = buf.clone();
            renderers.push(thread::spawn(move || {
                let mut rng = rand::thread_rng();
                for _ in 0..1000 {
                    let x = rng.gen_range(0..100);
                    let y = rng.gen_range(0..100);
                    r_buf.render(|buf| {
                        let _ = buf.get(x, y).unwrap();
                        // Assume 'front' buffer is being rendered/displayed
                    });
                }
            }));
        }

        // Spin up multiple update threads
        let mut workers = vec![];
        for _ in 0..5 {
            let mut w_buf = buf.clone();
            workers.push(thread::spawn(move || {
                let mut rng = rand::thread_rng();
                for _ in 0..1000 {
                    let x = rng.gen_range(0..100);
                    let y = rng.gen_range(0..100);
                    let c = rng.gen();
                    w_buf.update(|buf| {
                        buf.set(x, y, c).unwrap();
                    });
                }
            }));
        }

        // Join all threads to ensure they complete
        for renderer in renderers {
            renderer.join().unwrap();
        }

        for worker in workers {
            worker.join().unwrap();
        }
    }
}
