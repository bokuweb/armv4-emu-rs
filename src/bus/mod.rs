use super::types::{Byte, Word};

pub trait Bus {
    fn read_byte(&self, addr: u32) -> Byte;
    fn read_word(&self, addr: u32) -> Word;
    fn write_byte(&mut self, addr: u32, data: u8);
    fn write_word(&mut self, addr: u32, data: u32);
}
