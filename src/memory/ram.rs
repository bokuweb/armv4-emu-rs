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
