use constants::COND_FIELD;
use types::Word;

#[derive(Debug, PartialEq, Clone)]
pub enum Category {
    Undefined,
    Memory,
    DataProcessing,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Opcode {
    LDR,
    STR,
    MOV,
    // SWI,
    Undefined,
    DataProcessing,
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

#[derive(Debug)]
pub struct Instruction {
    pub cond: Condition,
    pub opcode: Opcode,
    pub category: Category,
    pub raw: Word,
}

pub const RAW_NOP: Word = 0b0000_00_0_1101_0_0000_0000_00000000_0000;

fn decode_memory_instruction(fetched: Word) -> Opcode {
    match fetched {
        v if (v & 0x0000_0800) == 0x0000_0000 => Opcode::LDR,
        v if (v & 0x0000_0800) == 0x0000_0800 => Opcode::STR,
        _ => panic!("unsupported instruction"),
    }
}

fn decode_data_processing_instruction(fetched: Word) -> Opcode {
    let cmd = (fetched & 0x01E0_0000) >> 21;
    match cmd {
        0b1101 => Opcode::MOV,
        _ => panic!("unsupported instruction"),
    }
}

impl Instruction {
    pub fn opcode(&self) -> Opcode {
        self.opcode.clone()
    }

    pub fn is_load(&self) -> bool {
        0 != (self.raw & (1 << 11))
    }

    pub fn decode(fetched: Word) -> Instruction {
        let cond = fetched & COND_FIELD;
        let cond = match cond {
            COND_AL => Condition::AL,
            _ => panic!("Unknowm condition {}", cond),
        };

        let category = match fetched {
            v if (v & 0x0E00_0010) == 0x0600_0010 => Category::Undefined,
            v if (v & 0x0C00_0000) == 0x0400_0000 => Category::Memory,
            v if (v & 0x0C00_0000) == 0x0000_0000 => Category::DataProcessing,
            // v if (v & 0x0F00_0000) == 0x0F00_0000 => Category::SWI,
            _ => panic!("Unsupported instruction"),
        };

        let opcode = match category {
            Category::Undefined => Opcode::Undefined,
            Category::Memory => decode_memory_instruction(fetched),
            Category::DataProcessing => decode_data_processing_instruction(fetched),
            // v if (v & 0x0F00_0000) == 0x0F00_0000 => Opcode::SWI,
            _ => panic!("unsupported instruction"),
        };

        Instruction {
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

    #[allow(non_snake_case)]
    pub fn get_src2(&self) -> usize {
        (self.raw as usize) & 0x0fff
    }

    pub fn has_I(&self) -> bool {
        self.raw & 0x0200_0000 != 0
    }

    pub fn is_plus_offset(&self) -> bool {
        self.raw & 0x0080_0000 == 0
    }

    pub fn is_minus_offset(&self) -> bool {
        !self.is_plus_offset()
    }
}
