use bytes::buf::BufMut;
use bytes::{Bytes, BytesMut};
use std::sync::{Arc, Mutex};


#[derive(Clone)]
pub struct Buffer {
    buffer: Arc<Mutex<BytesMut>>,
}

impl Buffer {
    pub fn new(size: usize) -> Buffer {
        let buffer = BytesMut::with_capacity(size);
        Buffer {
            buffer: Arc::new(Mutex::new(buffer))
        }
    }

    pub fn available(&self) -> usize {
        self.buffer.lock().unwrap().len()
    }

    pub fn read(&self, size: usize) -> Bytes {
        self.buffer.lock().unwrap().split_to(size).freeze()
    }

    pub fn reserve(&self, size: usize) -> usize {
        let mut buf = self.buffer.lock().unwrap();
        buf.reserve(size);
        buf.remaining_mut()
    }

    pub fn write(&self, bytes: &[u8]) {
        let mut buf = self.buffer.lock().unwrap();
        buf.put(bytes);
    }
}
