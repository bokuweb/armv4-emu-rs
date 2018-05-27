use constants::COND_FIELD;
use types::{Shift, Word};

#[derive(Debug, PartialEq, Clone)]
pub enum Category {
    Undefined,
    Multiple,
    Memory,
    DataProcessing,
    Branch,
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
pub struct Decoder {
    pub cond: Condition,
    pub opcode: Opcode,
    pub category: Category,
    pub raw: Word,
}

#[derive(Debug)]
pub struct MultipleDecoder {
    pub cond: Condition,
    pub opcode: Opcode,
    pub category: Category,
    pub raw: Word,
}

pub const RAW_NOP: Word = 0b0000_00_0_1101_0_0000_0000_00000000_0000;

fn get_I(raw: Word) -> Word {
    (raw & 0x0200_0000) >> 25
}

fn get_sh(raw: Word) -> Word {
    (raw & 0b11_0_0000) >> 5
}

fn get_instr(raw: Word) -> Word {
    (raw & 0b1111_1111_0000) >> 4
}

fn get_S(raw: Word) -> Word {
    (raw & 0x0010_0000) >> 20
}

pub trait Raw {
    fn raw(&self) -> u32;
    fn op(&self) -> Opcode;
}

pub trait BaseDecoder: Raw {
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
    fn get_Ra(&self) -> usize;

    fn get_src2(&self) -> usize {
        (self.raw() as usize) & 0x0fff
    }

    fn get_imm24(&self) -> i32 {
        self.raw() as i32 & 0xff_ffff
    }

    fn has_I(&self) -> bool {
        self.raw() & 0x0200_0000 != 0
    }

    fn is_pre_indexed(&self) -> bool {
        (self.raw() & (1 << 24)) != 0
    }

    fn is_write_back(&self) -> bool {
        (self.raw() & (1 << 21)) != 0
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

impl BaseDecoder for Decoder {
    #[allow(non_snake_case)]
    fn get_Ra(&self) -> usize {
        panic!("Ra field is not supported in default decoder");
    }
}

impl BaseDecoder for MultipleDecoder {
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

impl Raw for Decoder {
    fn raw(&self) -> u32 {
        self.raw
    }

    fn op(&self) -> Opcode {
        self.opcode.clone()
    }
}

impl Raw for MultipleDecoder {
    fn raw(&self) -> u32 {
        self.raw
    }

    fn op(&self) -> Opcode {
        self.opcode.clone()
    }
}

impl Decoder {
    fn decode_multiple(fetched: Word) -> Opcode {
        let cmd = (fetched & 0x01E0_0000) >> 21;
        match cmd {
            0b0000 => Opcode::MUL,
            0b0001 => Opcode::MLA,
            0b0100 => Opcode::UMULL,
            0b0101 => Opcode::UMLAL,
            _ => unimplemented!(),
        }
    }

    fn decode_memory(fetched: Word) -> Opcode {
        match fetched {
            v if (v & 0x0050_0000) == 0x0050_0000 => Opcode::LDRB,
            v if (v & 0x0010_0000) == 0x0010_0000 => Opcode::LDR,
            v if (v & 0x0040_0000) == 0x0040_0000 => Opcode::STRB,
            _ => Opcode::STR,
        }
    }

    #[allow(non_snake_case)]
    fn decode_data_processing(fetched: Word) -> Opcode {
        let cmd = (fetched & 0x01E0_0000) >> 21;
        let S = get_S(fetched) != 0;
        let I = get_I(fetched) != 0;
        let sh = get_sh(fetched);
        let instr = get_instr(fetched);
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

    fn decode_branch(fetched: Word) -> Opcode {
        let with_link = fetched & 0x0100_0000 != 0;
        if with_link {
            Opcode::BL
        } else {
            Opcode::B
        }
    }

    pub fn decode(fetched: Word) -> Box<BaseDecoder> {
        let cond = fetched & COND_FIELD;
        let cond = match cond {
            COND_AL => Condition::AL,
            _ => panic!("Unknowm condition {}", cond),
        };

        let category = match fetched {
            v if (v & 0x0E00_0000) == 0x0A00_0000 => Category::Branch,
            v if (v & 0x0FC0_00F0) == 0x0000_0090 => Category::Multiple,
            v if (v & 0x0F80_00F0) == 0x0080_0090 => Category::Multiple,
            v if (v & 0x0E00_0010) == 0x0600_0010 => Category::Undefined,
            v if (v & 0x0C00_0000) == 0x0400_0000 => Category::Memory,
            v if (v & 0x0C00_0000) == 0x0000_0000 => Category::DataProcessing,
            // v if (v & 0x0F00_0000) == 0x0F00_0000 => Category::SWI,
            _ => panic!("Unsupported instruction"),
        };

        let opcode = match category {
            Category::Undefined => Opcode::Undefined,
            Category::Multiple => Decoder::decode_multiple(fetched),
            Category::Memory => Decoder::decode_memory(fetched),
            Category::DataProcessing => Decoder::decode_data_processing(fetched),
            Category::Branch => Decoder::decode_branch(fetched),
            // v if (v & 0x0F00_0000) == 0x0F00_0000 => Opcode::SWI,
            _ => panic!("unsupported instruction"),
        };

        match category {
            Category::Multiple => Box::new(MultipleDecoder {
                raw: fetched,
                cond,
                category,
                opcode,
            }),
            _ => Box::new(Decoder {
                raw: fetched,
                cond,
                category,
                opcode,
            }),
        }
    }
}
