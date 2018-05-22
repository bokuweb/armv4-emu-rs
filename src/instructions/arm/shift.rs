use types::*;

pub fn shift(shift_type: Shift, value: u32, shift: u32) -> u32 {
    match shift_type {
        Shift::LSL => lsl(value, shift),
        Shift::LSR => lsr(value, shift),
        Shift::ASR => asr(value, shift),
        Shift::ROR => ror(value, shift),
    }
}

pub fn is_carry_over(shift_type: Shift, value: u32, shift: u32) -> Option<bool> {
    if shift == 0 {
        None
    } else {
        match shift_type {
            Shift::LSL => Some(value & (1 << (32 - shift)) != 0),
            _ => Some(value & (1 << (shift - 1)) != 0),
        }
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
    if shift == 0 {
        return value;
    }
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

#[test]
fn test_carry_lsl() {
    assert_eq!(is_carry_over(Shift::LSL, 0x8000_0000, 1), Some(true));
}

#[test]
fn test_without_carry_lsl() {
    assert_eq!(is_carry_over(Shift::LSL, 0x8000_0000, 2), Some(false));
}

#[test]
fn test_carry_ror() {
    assert_eq!(is_carry_over(Shift::ROR, 0x0000_0001, 1), Some(true));
}

#[test]
fn test_without_carry_ror() {
    assert_eq!(is_carry_over(Shift::ROR, 0x0000_0001, 2), Some(false));
}
