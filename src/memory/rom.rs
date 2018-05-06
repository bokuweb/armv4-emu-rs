use super::readable::*;
use super::Raw;

#[derive(Debug)]
pub struct Rom(Vec<u8>);

impl Rom {
    pub fn new(buf: Vec<u8>) -> Self {
        Rom(buf.clone())
    }
}

impl Raw for Rom {
    fn raw(&self, addr: u32) -> &[u8] {
        &self.0[(addr as usize)..]
    }
}

impl ByteReadable for Rom {}
impl HalfWordReadable for Rom {}
impl WordReadable for Rom {}
