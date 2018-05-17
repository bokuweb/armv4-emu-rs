use constants::COND_FIELD;
use types::{Shift, Word};

#[derive(Debug, PartialEq, Clone)]
pub enum Category {
    Undefined,
    Memory,
    DataProcessing,
    Branch,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Opcode {
    LDR,
    LDRB,
    STR,
    STRB,
    AND,
    EOR,
    SUB,
    RSB,
    ADD,
    ADC,
    SBC,
    RSC,
    MOV,
    B,
    BL,
    Undefined,
    // DataProcessing,
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

pub const RAW_NOP: Word = 0b0000_00_0_1101_0_0000_0000_00000000_0000;

impl Decoder {
    pub fn opcode(&self) -> Opcode {
        self.opcode.clone()
    }

    fn decode_memory(fetched: Word) -> Opcode {
        match fetched {
            v if (v & 0x0050_0000) == 0x0050_0000 => Opcode::LDRB,
            v if (v & 0x0010_0000) == 0x0010_0000 => Opcode::LDR,
            v if (v & 0x0040_0000) == 0x0040_0000 => Opcode::STRB,
            _ => Opcode::STR,
        }
    }

    fn decode_data_processing(fetched: Word) -> Opcode {
        let cmd = (fetched & 0x01E0_0000) >> 21;
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
            0b1101 => Opcode::MOV,
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

    pub fn decode(fetched: Word) -> Decoder {
        let cond = fetched & COND_FIELD;
        let cond = match cond {
            COND_AL => Condition::AL,
            _ => panic!("Unknowm condition {}", cond),
        };

        let category = match fetched {
            v if (v & 0x0E00_0000) == 0x0A00_0000 => Category::Branch,
            v if (v & 0x0E00_0010) == 0x0600_0010 => Category::Undefined,
            v if (v & 0x0C00_0000) == 0x0400_0000 => Category::Memory,
            v if (v & 0x0C00_0000) == 0x0000_0000 => Category::DataProcessing,
            // v if (v & 0x0F00_0000) == 0x0F00_0000 => Category::SWI,
            _ => panic!("Unsupported instruction"),
        };

        let opcode = match category {
            Category::Undefined => Opcode::Undefined,
            Category::Memory => Decoder::decode_memory(fetched),
            Category::DataProcessing => Decoder::decode_data_processing(fetched),
            Category::Branch => Decoder::decode_branch(fetched),
            // v if (v & 0x0F00_0000) == 0x0F00_0000 => Opcode::SWI,
            _ => panic!("unsupported instruction"),
        };

        Decoder {
            raw: fetched,
            cond,
            category,
            opcode,
        }
    }

    #[allow(non_snake_case)]
    pub fn get_Rn(&self) -> usize {
        (self.raw as usize >> 16) & 0b1111
    }

    #[allow(non_snake_case)]
    pub fn get_Rd(&self) -> usize {
        (self.raw as usize >> 12) & 0b1111
    }

    pub fn get_src2(&self) -> usize {
        (self.raw as usize) & 0x0fff
    }

    pub fn get_imm24(&self) -> i32 {
        self.raw as i32 & 0xff_ffff
    }

    pub fn has_I(&self) -> bool {
        self.raw & 0x0200_0000 != 0
    }

    pub fn is_pre_indexed(&self) -> bool {
        (self.raw & (1 << 24)) != 0
    }

    pub fn is_write_back(&self) -> bool {
        (self.raw & (1 << 21)) != 0
    }

    #[allow(non_snake_case)]
    pub fn get_memory_index_mode(&self) -> IndexMode {
        let P = (self.raw & (1 << 24)) != 0;
        let W = (self.raw & (1 << 21)) != 0;
        match (P, W) {
            (false, false) => IndexMode::PostIndex,
            (false, true) => IndexMode::Unsupported,
            (true, false) => IndexMode::Offset,
            (true, true) => IndexMode::PreIndex,
        }
    }

    pub fn get_Rm(&self) -> u32 {
        self.raw & 0b1111
    }

    pub fn get_sh(&self) -> Shift {
        match (self.raw & 0b11_0_0000) >> 5 {
            0b00 => Shift::LSL,
            0b01 => Shift::LSR,
            0b10 => Shift::ASR,
            0b11 => Shift::ROR,
            _ => unreachable!(),
        }
    }

    #[allow(non_snake_case)]
    pub fn get_Rs(&self) -> u32 {
        (self.raw & 0b1111_0_00_0_0000) >> 8
    }

    pub fn get_shamt5(&self) -> u32 {
        (self.raw & 0b11111_00_0_0000) >> 7
    }

    pub fn get_imm8(&self) -> u32 {
        self.raw & 0xFF
    }

    pub fn get_rot(&self) -> u32 {
        (self.raw & 0x0000_0F_00) >> 8
    }

    // pub fn has_B(&self) -> bool {
    //     self.raw & 0x0040_0000 != 0
    // }

    pub fn is_plus_offset(&self) -> bool {
        self.raw & 0x0080_0000 != 0
    }

    pub fn is_reg_offset(&self) -> bool {
        self.raw & 0x0000_0010 != 0
    }

    pub fn is_minus_offset(&self) -> bool {
        !self.is_plus_offset()
    }

    // fn is_branch_with_link(&self) -> bool {
    //     self.raw & 0x0100_0000 != 0
    // }
}
