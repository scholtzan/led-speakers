use bytes::buf::BufMut;
use bytes::{Bytes, BytesMut};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
/// Thread-safe bytes buffer.
pub struct Buffer {
    buffer: Arc<Mutex<BytesMut>>,
}

impl Buffer {
    /// Creates a new buffer instance with the provided size.
    ///
    /// # Examples
    /// ```
    /// let buffer = Buffer::new(1000);
    /// ```
    pub fn new(size: usize) -> Buffer {
        let buffer = BytesMut::with_capacity(size);
        Buffer {
            buffer: Arc::new(Mutex::new(buffer)),
        }
    }

    /// Returns how many bytes are still available in the buffer.
    ///
    /// # Examples
    /// ```
    /// let buffer = Buffer::new(1000);
    /// let available = buffer.available();
    /// ```
    pub fn available(&self) -> usize {
        self.buffer.lock().unwrap().len()
    }

    /// Read provided number of bytes from the buffer.
    ///
    /// # Examples
    /// ```
    /// let buffer = Buffer::new(1000);
    /// let read = buffer.read(10);
    /// ```
    pub fn read(&self, size: usize) -> Bytes {
        self.buffer.lock().unwrap().split_to(size).freeze()
    }

    /// Reserve provided number of bytes in the buffer.
    ///
    /// # Examples
    /// ```
    /// let buffer = Buffer::new(1000);
    /// let reserved = buffer.reserve(10);
    /// ```
    pub fn reserve(&self, size: usize) -> usize {
        let mut buf = self.buffer.lock().unwrap();
        buf.reserve(size);
        buf.remaining_mut()
    }

    /// Writes provided bytes into buffer.
    ///
    /// # Examples
    /// ```
    /// let buffer = Buffer::new(1000);
    /// let bytes: [u8; 10] = [0; 10];
    /// buffer.write(&bytes);
    /// ```
    ///
    pub fn write(&self, bytes: &[u8]) {
        let mut buf = self.buffer.lock().unwrap();
        buf.put(bytes);
    }
}
