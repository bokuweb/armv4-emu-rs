use constants::COND_FIELD;
use types::{Shift, Word};

#[derive(Debug, PartialEq, Clone)]
pub enum Category {
    Undefined,
    Multiple,
    Memory,
    ExtraMemory,
    DataProcessing,
    Branch,
    MultiLoadAndStore,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Opcode {
    AND,
    EOR,
    SUB,
    RSB,
    ADD,
    ADC,
    SBC,
    RSC,
    TST,
    TEQ,
    CMP,
    CMN,
    ORR,
    MOV,
    LSL,
    LSR,
    ASR,
    RRX,
    ROR,
    BIC,
    MVN,
    MUL,
    MLA,
    UMULL,
    UMLAL,
    SMULL,
    SMLAL,
    STR,
    LDR,
    LDRB,
    STRB,
    STRH,
    LDRH,
    LDRSB,
    LDRSH,
    B,
    BL,
    LDM,
    STM,
    Undefined,
    // SWI,
    NOP,
}

#[derive(Debug, PartialEq)]
pub enum Condition {
    EQ,
    NE,
    CS_HS,
    CC_LO,
    MI,
    PL,
    VS,
    VC,
    HI,
    LS,
    GE,
    LT,
    GT,
    LE,
    AL,
}

#[derive(Debug, PartialEq, Clone)]
pub enum IndexMode {
    PostIndex,
    Unsupported,
    Offset,
    PreIndex,
}

#[derive(Debug)]
pub struct BaseDecoder {
    pub cond: Condition,
    pub opcode: Opcode,
    // pub category: Category,
    pub raw: Word,
}

#[derive(Debug)]
pub struct MultipleDecoder(BaseDecoder);
#[derive(Debug)]
pub struct ExtraMemoryDecoder(BaseDecoder);
// #[derive(Debug)]
// pub struct MultiLoadAndStoreDecoder(BaseDecoder);

// pub const RAW_NOP: Word = 0b0000_00_0_1101_0_0000_0000_00000000_0000;

pub fn is_load(raw: Word) -> bool {
    raw & 0x0010_0000 != 0
}

#[allow(non_snake_case)]
fn get_I(raw: Word) -> Word {
    (raw & 0x0200_0000) >> 25
}

fn get_sh(raw: Word) -> Word {
    (raw & 0b11_0_0000) >> 5
}

fn get_instr(raw: Word) -> Word {
    (raw & 0b1111_1111_0000) >> 4
}

#[allow(non_snake_case)]
fn get_S(raw: Word) -> Word {
    (raw & 0x0010_0000) >> 20
}

pub trait Raw {
    fn raw(&self) -> u32;
    fn op(&self) -> Opcode;
}

pub trait Decoder: Raw {
    fn opcode(&self) -> Opcode {
        self.op()
    }

    #[allow(non_snake_case)]
    fn get_Rn(&self) -> usize {
        (self.raw() as usize >> 16) & 0b1111
    }

    #[allow(non_snake_case)]
    fn get_Rd(&self) -> usize {
        (self.raw() as usize >> 12) & 0b1111
    }

    #[allow(non_snake_case)]
    fn get_Ra(&self) -> usize {
        panic!("Ra field is not supported in default decoder");
    }

    fn get_src2(&self) -> usize {
        (self.raw() as usize) & 0x0fff
    }

    fn get_imm24(&self) -> i32 {
        self.raw() as i32 & 0xff_ffff
    }

    #[allow(non_snake_case)]
    fn has_I(&self) -> bool {
        self.raw() & 0x0200_0000 != 0
    }

    fn is_pre_indexed(&self) -> bool {
        (self.raw() & (1 << 24)) != 0
    }

    fn is_write_back(&self) -> bool {
        (self.raw() & (1 << 21)) != 0
    }

    fn is_load(&self) -> bool {
        is_load(self.raw())
    }

    #[allow(non_snake_case)]
    fn get_memory_index_mode(&self) -> IndexMode {
        let P = (self.raw() & (1 << 24)) != 0;
        let W = (self.raw() & (1 << 21)) != 0;
        match (P, W) {
            (false, false) => IndexMode::PostIndex,
            (false, true) => IndexMode::Unsupported,
            (true, false) => IndexMode::Offset,
            (true, true) => IndexMode::PreIndex,
        }
    }

    #[allow(non_snake_case)]
    fn get_Rm(&self) -> usize {
        self.raw() as usize & 0b1111
    }

    fn get_sh(&self) -> Shift {
        match (self.raw() & 0b11_0_0000) >> 5 {
            0b00 => Shift::LSL,
            0b01 => Shift::LSR,
            0b10 => Shift::ASR,
            0b11 => Shift::ROR,
            _ => unreachable!(),
        }
    }

    #[allow(non_snake_case)]
    fn get_Rs(&self) -> u32 {
        (self.raw() & 0b1111_0_00_0_0000) >> 8
    }

    fn get_shamt5(&self) -> u32 {
        (self.raw() & 0b11111_00_0_0000) >> 7
    }

    fn get_imm8(&self) -> u32 {
        self.raw() & 0xFF
    }

    fn get_rot(&self) -> u32 {
        (self.raw() & 0x0000_0F_00) >> 8
    }

    // pub fn has_B(&self) -> bool {
    //     self.raw & 0x0040_0000 != 0
    // }

    // Bit: 23
    fn is_plus_offset(&self) -> bool {
        self.raw() & 0x0080_0000 != 0
    }

    fn is_reg_offset(&self) -> bool {
        self.raw() & 0x0000_0010 != 0
    }

    fn is_minus_offset(&self) -> bool {
        !self.is_plus_offset()
    }

    // fn is_branch_with_link(&self) -> bool {
    //     self.raw & 0x0100_0000 != 0
    // }
}

impl Decoder for BaseDecoder {}

// impl Decoder for MultiLoadAndStoreDecoder {}

impl Decoder for MultipleDecoder {
    #[allow(non_snake_case)]
    fn get_Ra(&self) -> usize {
        (self.raw() as usize >> 12) & 0b1111
    }

    #[allow(non_snake_case)]
    fn get_Rd(&self) -> usize {
        (self.raw() as usize >> 16) & 0b1111
    }

    #[allow(non_snake_case)]
    fn get_Rn(&self) -> usize {
        self.raw() as usize & 0b1111
    }

    fn get_Rm(&self) -> usize {
        (self.raw() >> 8) as usize & 0b1111
    }
}

impl Decoder for ExtraMemoryDecoder {
    #[allow(non_snake_case)]
    fn has_I(&self) -> bool {
        self.raw() & 0x0040_0000 != 0
    }

    fn get_imm8(&self) -> u32 {
        (self.raw() & 0xF00).wrapping_shr(4) + (self.raw() & 0xF)
    }
}

// impl MultiLoadAndStoreDecoder {
//     #[allow(non_snake_case)]
//     fn get_register_list(&self) -> usize {
//         (self.raw() & 0xFFFF) as usize
//     }
// }

impl Raw for BaseDecoder {
    fn raw(&self) -> u32 {
        self.raw
    }

    fn op(&self) -> Opcode {
        self.opcode.clone()
    }
}

impl Raw for MultipleDecoder {
    fn raw(&self) -> u32 {
        self.0.raw
    }

    fn op(&self) -> Opcode {
        self.0.opcode.clone()
    }
}

impl Raw for ExtraMemoryDecoder {
    fn raw(&self) -> u32 {
        self.0.raw
    }

    fn op(&self) -> Opcode {
        self.0.opcode.clone()
    }
}

// impl Raw for MultiLoadAndStoreDecoder {
//     fn raw(&self) -> u32 {
//         self.0.raw
//     }
//
//     fn op(&self) -> Opcode {
//         self.0.opcode.clone()
//     }
// }

fn decode_multiple(raw: Word) -> Opcode {
    let cmd = (raw & 0x01E0_0000) >> 21;
    match cmd {
        0b0000 => Opcode::MUL,
        0b0001 => Opcode::MLA,
        0b0100 => Opcode::UMULL,
        0b0101 => Opcode::UMLAL,
        0b0110 => Opcode::SMULL,
        0b0111 => Opcode::SMLAL,
        _ => unimplemented!(),
    }
}

fn decode_memory(raw: Word) -> Opcode {
    match raw {
        v if (v & 0x0050_0000) == 0x0050_0000 => Opcode::LDRB,
        v if (v & 0x0010_0000) == 0x0010_0000 => Opcode::LDR,
        v if (v & 0x0040_0000) == 0x0040_0000 => Opcode::STRB,
        _ => Opcode::STR,
    }
}

fn decode_extra_memory(raw: Word) -> Opcode {
    let op2 = (raw >> 5) & 0b11;
    let l = is_load(raw);
    match op2 {
        0b01 if !l => Opcode::STRH,
        0b01 if l => Opcode::LDRH,
        0b10 if l => Opcode::LDRSB,
        0b11 if l => Opcode::LDRSH,
        _ => panic!("undefined instruction detected"),
    }
}

fn decode_data_processing(raw: Word) -> Opcode {
    let cmd = (raw & 0x01E0_0000) >> 21;
    let S = get_S(raw) != 0;
    let I = get_I(raw) != 0;
    let sh = get_sh(raw);
    let instr = get_instr(raw);
    debug!("data processing cmd = {:x}", cmd);
    match cmd {
        0b0000 => Opcode::AND,
        0b0001 => Opcode::EOR,
        0b0010 => Opcode::SUB,
        0b0011 => Opcode::RSB,
        0b0100 => Opcode::ADD,
        0b0101 => Opcode::ADC,
        0b0110 => Opcode::SBC,
        0b0111 => Opcode::RSC,
        0b1000 if S => Opcode::TST,
        0b1001 if S => Opcode::TEQ,
        0b1010 if S => Opcode::CMP,
        0b1011 if S => Opcode::CMN,
        0b1100 => Opcode::ORR,
        0b1101 if I || instr == 0 => Opcode::MOV,
        0b1101 if !I && sh == 0b00 => Opcode::LSL,
        0b1101 if !I && sh == 0b01 => Opcode::LSR,
        0b1101 if !I && sh == 0b10 => Opcode::ASR,
        0b1101 if !I && sh == 0b11 && (instr & 0xF90) == 0 => Opcode::RRX,
        0b1101 if !I && sh == 0b11 && instr != 0 => Opcode::ROR,
        0b1110 => Opcode::BIC,
        0b1111 => Opcode::MVN,
        _ => panic!("unsupported instruction"),
    }
}

fn decode_multi_load_and_store(raw: Word) -> Opcode {
    if is_load(raw) {
        Opcode::LDM
    } else {
        Opcode::STM
    }
}

fn decode_branch(raw: Word) -> Opcode {
    let with_link = raw & 0x0100_0000 != 0;
    if with_link {
        Opcode::BL
    } else {
        Opcode::B
    }
}

pub fn decode(raw: Word) -> Box<Decoder> {
    let cond = raw & COND_FIELD;
    let cond = match cond {
        COND_AL => Condition::AL,
        _ => panic!("Unknowm condition {}", cond),
    };

    let category = match raw {
        v if (v & 0x0E00_0000) == 0x0A00_0000 => Category::Branch,
        v if (v & 0x0FC0_00F0) == 0x0000_0090 => Category::Multiple,
        v if (v & 0x0F80_00F0) == 0x0080_0090 => Category::Multiple,
        v if (v & 0x0E00_0010) == 0x0600_0010 => Category::Undefined,
        v if (v & 0x0E40_0F90) == 0x0000_0090 => Category::ExtraMemory,
        v if (v & 0x0E40_0090) == 0x0040_0090 => Category::ExtraMemory,
        v if (v & 0x0C00_0000) == 0x0400_0000 => Category::Memory,
        v if (v & 0x0C00_0000) == 0x0000_0000 => Category::DataProcessing,
        v if (v & 0x0E00_0000) == 0x0800_0000 => Category::MultiLoadAndStore, // LDM and STM,
        // v if (v & 0x0F00_0000) == 0x0F00_0000 => Category::SWI,
        _ => panic!("Unsupported instruction"),
    };

    let opcode = match category {
        Category::Undefined => Opcode::Undefined,
        Category::Multiple => decode_multiple(raw),
        Category::Memory => decode_memory(raw),
        Category::ExtraMemory => decode_extra_memory(raw),
        Category::DataProcessing => decode_data_processing(raw),
        Category::Branch => decode_branch(raw),
        Category::MultiLoadAndStore => decode_multi_load_and_store(raw),
        // v if (v & 0x0F00_0000) == 0x0F00_0000 => Opcode::SWI,
        _ => panic!("unsupported instruction"),
    };

    debug!("opcode = {:?}", opcode);
    let dec = BaseDecoder { raw, cond, opcode };
    match category {
        Category::Multiple => Box::new(MultipleDecoder(dec)),
        Category::ExtraMemory => Box::new(ExtraMemoryDecoder(dec)),
        _ => Box::new(dec),
    }
}
