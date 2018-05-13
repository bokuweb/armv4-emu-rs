extern crate env_logger;
extern crate goblin;
#[macro_use]
extern crate log;
extern crate byteorder;

mod constants;
mod core;
mod instructions;
mod decoder;
mod registers;
mod types;
mod memory;

use constants::*;
use std::cell::RefCell;
use std::rc::Rc;
use types::*;
use memory::readable::*;
use memory::writable::*;
use memory::rom::Rom;
use memory::ram::Ram;

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

struct CpuBus {
    rom: Rc<RefCell<Rom>>,
    ram: Rc<RefCell<Ram>>,
}

impl core::Bus for CpuBus {
    fn read_byte(&self, addr: u32) -> Byte {
        match addr {
            0x0000_0000...0x0007_FFFF => self.rom.borrow().read_byte(addr),
            _ => panic!("TODO: "),
        }
    }    
    fn read_word(&self, addr: u32) -> Word {
        match addr {
            0x0000_0000...0x0007_FFFF => self.rom.borrow().read_word(addr),
            _ => panic!("TODO: "),
        }
    }
}

impl CpuBus {
    fn new(rom: Rc<RefCell<Rom>>, ram: Rc<RefCell<Ram>>) -> CpuBus {
        CpuBus { rom, ram }
    }
}

fn load_bin(bin: String) -> Result<Vec<u8>, std::io::Error> {
    let path = Path::new(&bin);
    let mut fd = File::open(path)?;
    let mut buf = Vec::new();
    fd.read_to_end(&mut buf)?;
    Ok(buf)
}


fn main() {
    env_logger::init();
    // let elf_path = env::args().nth(1).expect("");
    // let result = load_elf(elf_path);
    let bin_path = env::args()
        .nth(1)
        .expect("Specify bin filename to build.");
    let bin = load_bin(bin_path).expect("faild to read bin");
    debug!("read bin data = {:?}", bin);
    let rom = memory::rom::Rom::new(0x80000, bin);
    let ram = memory::ram::Ram::new(vec![0; 0x10000]);
    let bus = CpuBus::new(Rc::new(RefCell::new(rom)), Rc::new(RefCell::new(ram)));
    let mut arm = core::ARMv4::new(Rc::new(RefCell::new(bus)));
    arm.tick();
    println!("{:?}", arm.get_gpr(0));
    arm.tick();
    arm.tick();
    arm.tick();
    arm.tick();
    arm.tick();
    arm.tick();
    arm.tick();
    arm.tick();
    arm.tick();
    arm.tick();
    arm.tick();
    arm.tick();
}
