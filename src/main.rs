extern crate env_logger;
extern crate goblin;
#[macro_use]
extern crate log;
extern crate byteorder;

mod constants;
mod core;
mod instructions;
mod registers;
mod types;
mod memory;

use constants::*;
use std::cell::RefCell;
use std::rc::Rc;
use types::Word;
use memory::readable::*;
use memory::writable::*;
use memory::rom::Rom;
use memory::ram::Ram;

use goblin::{error, Object};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

struct CpuBus {
    rom: Rc<RefCell<Rom>>,
    ram: Rc<RefCell<Ram>>,
}

impl core::Bus for CpuBus {
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

fn memory_elf(elf_obj: goblin::elf::Elf, buffer: &Vec<u8>) -> HashMap<u64, u8> {
    let shdr_strtab = &elf_obj.shdr_strtab;
    println!("{:?}", &elf_obj);
    let mut memory = HashMap::new();
    for section in &elf_obj.section_headers {
        println!("elf_obj.section_headers = {:#?}, file_offset = {:#x}, size = {:#x}, type = {:#?} flags = {:#?}",
                 &shdr_strtab[section.sh_name],
                 section.sh_offset,
                 section.sh_size,
                 section.sh_type,
                 section.sh_flags);
        if section.sh_size != 0 {
            for idx in 0..(section.sh_size) {
                let mut offset = idx + section.sh_offset;
                if section.sh_type == 1 {
                    memory.insert(section.sh_addr + idx, buffer[offset as usize]);
                }
            }
        }
    }
    memory
}

fn dump_memory(memory: &HashMap<u64, u8>) {
    for (addr, data) in memory.iter() {
        println!("{:#x}: {:#02x}", addr, data);
    }
}

fn load_elf(hexfile: String) -> Result<(), goblin::error::Error> {
    let path = Path::new(&hexfile);
    let mut fd = File::open(path)?;
    let mut buffer = Vec::new();
    fd.read_to_end(&mut buffer)?;

    match Object::parse(&buffer)? {
        Object::Elf(elf) => {
            let mut memory = memory_elf(elf, &buffer);
            dump_memory(&mut memory)
        }
        _ => {
            println!("not supported.");
        }
    }
    Ok(())
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
