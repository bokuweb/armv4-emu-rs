extern crate env_logger;
extern crate goblin;

use goblin::{error, Object, elf};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn memory_elf(elf_obj: elf::Elf, buffer: &Vec<u8>) -> HashMap<u64, u8> {
    let shdr_strtab = &elf_obj.shdr_strtab;
    let mut memory = HashMap::new();
    for section in &elf_obj.section_headers {
        // println!("elf_obj.section_headers = {:#?}, file_offset = {:#x}, size = {:#x}, type = {:#?} flags = {:#?}",
        //          &shdr_strtab[section.sh_name],
        //          section.sh_offset,
        //          section.sh_size,
        //          section.sh_type,
        //          section.sh_flags);
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

fn load_elf(hexfile: String) -> Result<(), error::Error> {
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
