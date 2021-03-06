use std::cell::RefCell;
use std::rc::Rc;

use bus::Bus;
use constants::*;
use decoder::arm::Decoder;
use types::*;

use super::super::PipelineStatus;
use super::shift::shift;
use error::ArmError;

fn exec_memory_processing<F>(
    gpr: &mut [u32; 16],
    dec: &Decoder,
    load_or_store: F,
) -> Result<PipelineStatus, ArmError>
where
    F: Fn(&mut [u32; 16], u32),
{
    let mut base = gpr[dec.get_Rn()];
    // INFO: Treat as imm12 if not I.
    let offset = if !dec.has_I() {
        dec.get_src2() as u32
    } else {
        let rm = dec.get_Rm() as usize;
        let sh = dec.get_sh();
        let shamt5 = dec.get_shamt5();
        shift(sh, gpr[rm], shamt5)
    };
    let offset_base = if dec.is_plus_offset() {
        (base + offset) as Word
    } else {
        (base - offset) as Word
    };
    if dec.is_pre_indexed() {
        base = offset_base;
    }
    load_or_store(gpr, base);
    if !dec.is_pre_indexed() {
        gpr[dec.get_Rn()] = offset_base;
    } else if dec.is_write_back() {
        gpr[dec.get_Rn()] = base;
    }
    if dec.get_Rd() == PC && dec.is_load() {
        Ok(PipelineStatus::Flush)
    } else {
        Ok(PipelineStatus::Continue)
    }
}

#[allow(non_snake_case)]
pub fn exec_ldr<T>(
    bus: &Rc<RefCell<T>>,
    dec: &Decoder,
    gpr: &mut [Word; 16],
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_memory_processing(gpr, dec, |gpr, base| {
        gpr[dec.get_Rd()] = bus.borrow().read_word(base);
    })
}

#[allow(non_snake_case)]
pub fn exec_ldrb<T>(
    bus: &Rc<RefCell<T>>,
    dec: &Decoder,
    gpr: &mut [Word; 16],
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_memory_processing(gpr, dec, |gpr, base| {
        gpr[dec.get_Rd()] = bus.borrow().read_byte(base) as Word;
    })
}

pub fn exec_str<T>(
    bus: &Rc<RefCell<T>>,
    dec: &Decoder,
    gpr: &mut [Word; 16],
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_memory_processing(gpr, dec, |gpr, base| {
        bus.borrow_mut().write_word(base, gpr[dec.get_Rd()]);
    })
}

pub fn exec_strb<T>(
    bus: &Rc<RefCell<T>>,
    dec: &Decoder,
    gpr: &mut [Word; 16],
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_memory_processing(gpr, dec, |gpr, base| {
        bus.borrow_mut().write_byte(base, gpr[dec.get_Rd()] as Byte);
    })
}
