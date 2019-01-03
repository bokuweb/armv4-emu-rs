use super::readable::*;
use super::Raw;

#[derive(Debug)]
pub struct Rom(Vec<u8>);

impl Rom {
    pub fn new(size: usize, mut init: Vec<u8>) -> Self {
        let buf = vec![0; size];
        let len = init.len();
        init.extend(&buf[len..]);
        Rom(init)
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

#[test]
fn rom_read_byte() {
    let rom = Rom::new(4, vec![0x01, 0x00, 0x00, 0x00]);
    assert_eq!(rom.read_byte(0), 0x01);
}

#[test]
fn rom_read_halfword() {
    let rom = Rom::new(4, vec![0x01, 0x02, 0x00, 0x00]);
    assert_eq!(rom.read_halfword(0), 0x0201);
}

#[test]
fn rom_read_word() {
    let rom = Rom::new(4, vec![0x01, 0x02, 0x03, 0x04]);
    assert_eq!(rom.read_word(0), 0x0403_0201);
}
