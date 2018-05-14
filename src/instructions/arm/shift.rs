use types::*;

pub fn shift(shft_type: Shift, value: u32, shift: u32) -> u32 {
    match shft_type {
        Shift::LSL => lsl(value, shift),
        Shift::LSR => lsr(value, shift),
        Shift::ASR => asr(value, shift),
        Shift::ROR => ror(value, shift),
    }
}

pub fn lsl(value: u32, shift: u32) -> u32 {
    value << shift
}

pub fn lsr(value: u32, shift: u32) -> u32 {
    value >> shift
}

pub fn asr(value: u32, shift: u32) -> u32 {
    if value & (1 << 31) == 0 {
        value >> shift
    } else {
        value >> shift | (0xFFFF_FFFF << (32 - shift))
    }
}

pub fn ror(value: u32, shift: u32) -> u32 {
    (value >> shift) | (value << (32 - shift))
}

#[test]
fn test_ror() {
    assert_eq!(ror(0xA5A5_5A5A, 4), 0xAA5A_55A5);
}

#[test]
fn test_asr() {
    assert_eq!(ror(0xA5A5_5A5A, 4), 0xAA5A_55A5);
}
