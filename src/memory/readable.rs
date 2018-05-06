use super::Raw;
use byteorder::{ByteOrder, LittleEndian};

pub trait ByteReadable: Raw {
    fn read_byte(&self, addr: u32) -> u8 {
        self.raw(addr)[0]
    }
}

pub trait HalfWordReadable: Raw {
    fn read_halfword(&self, addr: u32) -> u16 {
        LittleEndian::read_u16(self.raw(addr))
    }
}

pub trait WordReadable: Raw {
    fn read_word(&self, addr: u32) -> u32 {
        LittleEndian::read_u32(self.raw(addr))
    }
}
