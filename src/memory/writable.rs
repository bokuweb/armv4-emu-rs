use super::MutRaw;
use byteorder::{ByteOrder, LittleEndian};

pub trait ByteWritable: MutRaw {
    fn write_byte(&mut self, addr: u32, data: u8) {
        self.mut_raw(addr)[0] = data;
    }
}

pub trait HalfWordWritable: MutRaw {
    fn write_halfword(&mut self, addr: u32, data: u16) {
        LittleEndian::write_u16(self.mut_raw(addr), data);
    }
}

pub trait WordWritable: MutRaw {
    fn write_word(&mut self, addr: u32, data: u32) {
        LittleEndian::write_u32(self.mut_raw(addr), data);
    }
}
