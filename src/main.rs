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

use constants::*;
use std::cell::RefCell;
use std::rc::Rc;
use types::Word;

use goblin::{error, Object};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

struct CpuBus;

impl core::Bus for CpuBus {
    fn read_word(&self, addr: u32) -> Word {
        0
    }
}

fn memory_elf(
    elf_obj: goblin::elf::Elf,
    memory: &mut std::collections::HashMap<u64, u8>,
    buffer: &std::vec::Vec<u8>,
) -> error::Result<()> {
    let shdr_strtab = &elf_obj.shdr_strtab;
    println!("{:?}", &elf_obj);
    for section in &elf_obj.section_headers {
        println!(
            "elf_obj.section_headers = {:#?}, file_offset = {:#x}, size = {:#x}",
            &shdr_strtab[section.sh_name], section.sh_offset, section.sh_size
        );
        if section.sh_size != 0 {
            for idx in 0..(section.sh_size) {
                let mut offset = idx + section.sh_offset;
                println!(
                    "adr {:X} idx {} data {:x}",
                    section.sh_addr, idx, buffer[offset as usize]
                );

                memory.insert(section.sh_addr + idx, buffer[offset as usize]);
            }
        }
    }
    Ok(())
}

fn dump_memory(memory: &std::collections::HashMap<u64, u8>) {
    for (addr, data) in memory.iter() {
        println!("{:#x}: {:#02x}", addr, data);
    }
}

fn load_elf(hexfile: String) -> error::Result<()> {
    let path = Path::new(&hexfile);
    let mut fd = File::open(path)?;
    let mut buffer = Vec::new();
    fd.read_to_end(&mut buffer)?;

    match Object::parse(&buffer)? {
        Object::Elf(elf) => {
            let mut memory = HashMap::new();
            // let mut my_number: () = memory;
            memory_elf(elf, &mut memory, &buffer);
            // dump_memory(&mut memory)
        }
        _ => {
            println!("not supported.");
        }
    }
    Ok(())
}

fn main() {
    match env::args().nth(1) {
        None => println!("Specify filename to build."),
        Some(arg) => {
            println!("{}", arg);
            let result = load_elf(arg);
        }
    }
    env_logger::init();
    let bus = CpuBus;
    let mut arm = core::ARMv4::new(Rc::new(RefCell::new(bus)));
    arm.tick();
}

/*

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
*/
