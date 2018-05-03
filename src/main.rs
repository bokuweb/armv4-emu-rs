extern crate env_logger;
#[macro_use]
extern crate log;
extern crate byteorder;

mod types;
mod core;
mod registers;
mod constants;
mod instructions;

use std::cell::RefCell;
use std::rc::Rc;
use constants::*;
use types::Word;

struct CpuBus;

impl core::Bus for CpuBus {
    fn read_word(&self, addr: u32) -> Word {
        0
    }
}

fn main() {
    env_logger::init();
    let bus = CpuBus;
    let mut arm = core::ARMv4::new(Rc::new(RefCell::new(bus)));
    arm.tick();
}
