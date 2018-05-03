
use types::Word;


#[allow(non_snake_case)]
#[derive(Debug)]
pub struct ARMRegisters {
    commons: Vec<Word>,
	// fiq: Vec<Word>,
}

#[allow(non_snake_case)]
pub trait Registers {
    fn get(&self, index: usize) -> Word;

    fn set(&self, index: usize, data: Word);
}

impl ARMRegisters {
    pub fn new() -> Self {
        ARMRegisters { commons: vec![0; 16] }
    }
}
