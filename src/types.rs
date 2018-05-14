pub type Byte = u8;
pub type HalfWord = u16;
pub type Word = u32;

#[derive(Debug, PartialEq, Clone)]
pub enum Shift {
    LSL,
    LSR,
    ASR,
    ROR,
}
