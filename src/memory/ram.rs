use super::readable::*;
use super::writable::*;
use super::Raw;
use super::MutRaw;

#[derive(Debug)]
pub struct Ram(Vec<u8>);

impl Ram {
    pub fn new(buf: Vec<u8>) -> Self {
        Ram(buf.clone())
    }
}

impl Raw for Ram {
    fn raw(&self, addr: u32) -> &[u8] {
        &self.0[(addr as usize)..]
    }
}

impl MutRaw for Ram {
    fn mut_raw(&mut self, addr: u32) -> &mut [u8] {
        &mut self.0[(addr as usize)..]
    }
}

impl ByteReadable for Ram {}
impl HalfWordReadable for Ram {}
impl WordReadable for Ram {}

impl ByteWritable for Ram {}
impl HalfWordWritable for Ram {}
impl WordWritable for Ram {}

#[test]
fn ram_read_byte() {
    let ram = Ram::new(vec![0x01, 0x00, 0x00, 0x00]);
    assert_eq!(ram.read_byte(0), 0x01);
}

#[test]
fn ram_read_halfword() {
    let ram = Ram::new(vec![0x01, 0x02, 0x00, 0x00]);
    assert_eq!(ram.read_halfword(0), 0x0201);
}

#[test]
fn ram_read_word() {
    let ram = Ram::new(vec![0x01, 0x02, 0x03, 0x04]);
    assert_eq!(ram.read_word(0), 0x0403_0201);
}

#[test]
fn ram_write_byte() {
    let mut ram = Ram::new(vec![0x00, 0x00, 0x00, 0x00]);
    ram.write_byte(0, 0x01);
    assert_eq!(ram.read_byte(0), 0x01);
}

#[test]
fn ram_write_halfword() {
    let mut ram = Ram::new(vec![0x00, 0x00, 0x00, 0x00]);
    ram.write_halfword(0, 0x1234);
    assert_eq!(ram.read_halfword(0), 0x1234);
}

#[test]
fn ram_write_word() {
    let mut ram = Ram::new(vec![0x00, 0x00, 0x00, 0x00]);
    ram.write_word(0, 0x1234_5678);
    assert_eq!(ram.read_word(0), 0x1234_5678);
}

