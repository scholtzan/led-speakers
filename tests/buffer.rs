use led_speakers::buffer::Buffer;

use std::str;

#[test]
fn buffer_write() {
    let buffer = Buffer::new(100);
    let bytes: [u8; 10] = [0; 10];
    buffer.write(&bytes);
    assert_eq!(buffer.available(), 10);
    buffer.write(&bytes);
    assert_eq!(buffer.available(), 20);
}

#[test]
fn buffer_reserve() {
    let buffer = Buffer::new(1000);
    let reserved = buffer.reserve(10);
    let bytes: [u8; 10] = [0; 10];
    buffer.write(&bytes);
    assert_eq!(buffer.reserve(10), reserved - 10);
}

#[test]
fn buffer_read() {
    let buffer = Buffer::new(1000);
    let bytes: [u8; 10] = [0; 10];
    buffer.write(&bytes);
    assert_eq!(buffer.read(10), str::from_utf8(&bytes).unwrap());
}
