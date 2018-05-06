pub mod rom;
pub mod ram;

pub mod readable;
pub mod writable;

pub trait Raw {
    fn raw(&self, offset: u32) -> &[u8];
}

pub trait MutRaw {
    fn mut_raw(&mut self, offset: u32) -> &mut [u8];
}