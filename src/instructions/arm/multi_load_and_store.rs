use std::cell::RefCell;
use std::rc::Rc;

use bus::Bus;
use constants::*;
use decoder::arm;
use types::*;

use super::super::PipelineStatus;
use error::ArmError;

// 31    28 27  25 24  23  22  21  20 19    16 15                      0
// ---------------------------------------------------------------------
// | cond | 1 0 0 | P | U | S | W | L |  Rn  |      Register LIst      |
// ---------------------------------------------------------------------
// P = 0: Post index 1: Pre index
// U = 0: Decrement 1: Increment
// S = Restore force user bit. S specifies if banked register access should occur when in privileged modes [or if R15 and 26 bit and user mode, if the PSR should be written while PC is updated]
// W = 1: Auto Index
// L = 0: Store / 1: Load
fn exec_multi_memory_processing<F>(
    gpr: &mut [u32; 16],
    dec: &arm::Decoder,
    load_or_store: F,
) -> Result<PipelineStatus, ArmError>
where
    F: Fn(&mut [u32; 16], u32, u32),
{
    let mut base: i64 = gpr[dec.get_Rn()] as i64;
    let register_map = dec.raw() & 0xFFFF;
    debug!("register map = {:x}", register_map);
    let offset: i64 = if dec.is_plus_offset() { 4 } else { -4 };
    println!("------- {:?}", gpr);
    for i in 0..0x10 {
        if register_map & (1 << i) != 0 {
            println!("------- {}", i);
            if dec.is_pre_indexed() {
                base = base.wrapping_add(offset);
            }
            load_or_store(gpr, base as u32, i);
            if !dec.is_pre_indexed() {
                base = base.wrapping_add(offset);
            }
        }
    }

    println!("------- {:?}", gpr);
    // TODO: Handle S flag.

    if dec.is_write_back() {
        println!("hogee {}", base);
        gpr[dec.get_Rn()] = base as u32;
    }

    // If PC is loaded
    if register_map & 0x8000 != 0 && dec.is_load() {
        Ok(PipelineStatus::Flush)
    } else {
        Ok(PipelineStatus::Continue)
    }
}

pub fn exec_ldm<T>(
    bus: &Rc<RefCell<T>>,
    dec: &arm::Decoder,
    gpr: &mut [Word; 16],
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_multi_memory_processing(gpr, dec, |gpr, base, i| {
        gpr[i as usize] = bus.borrow().read_word(base) as Word;
    })
}

pub fn exec_stm<T>(
    bus: &Rc<RefCell<T>>,
    dec: &arm::Decoder,
    gpr: &mut [Word; 16],
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_multi_memory_processing(gpr, dec, |gpr, base, i| {
        bus.borrow_mut().write_word(base, gpr[i as usize] as Word);
    })
}

/*
fn execute_ldm_stm(&mut self, inst: ArmInstruction) -> Result<CpuAction, GbaError> {
    // TODO Handle store/load base as first or later register.
    let base  = self.gpr[inst.Rn()] as u32;
    let rmap  = inst.register_map();
    let bytes = 4 * rmap.count_ones();
    let r15   = 0 != (rmap & 0x8000);
    let psr   = inst.is_enforcing_user_mode();
    let offs  = if inst.is_pre_indexed() == inst.is_offset_added() { (4_u32, 0) } else { (0_u32, 4) };
    let mut addr = if inst.is_offset_added() { base } else { base.wrapping_sub(bytes) }; // Go back N regs if decr.

    // Write back Rn now to avoid special cases with loading Rn.
    if inst.is_auto_incrementing() {
        self.gpr[inst.Rn()] = if inst.is_offset_added() { base.wrapping_add(bytes) as i32 } else { base.wrapping_sub(bytes) as i32 };
    }

    // Handle privileged transfers.
    if psr & !(r15 & inst.is_load()) {
        if self.mode == Mode::User { return Err(GbaError::PrivilegedUserCode); }
        try!(self.execute_ldm_stm_user_bank(rmap, addr, offs, inst.is_load()));
        if inst.is_auto_incrementing() { warn!("W-bit set for LDM/STM with PSR transfer/USR banks."); }
    } else {
        for i in 0_u32..16 { if 0 != (rmap & (1 << i)) {
            addr = addr.wrapping_add(offs.0);
            if inst.is_load() { self.gpr[i as usize] = try!(self.bus.borrow().load_word(addr)); }
            else              { try!(self.bus.borrow_mut().store_word(addr, self.gpr[i as usize])); }
            addr = addr.wrapping_add(offs.1);
        }}
    }

    // Handle mode change.
    if r15 & psr & inst.is_load() {
        if self.mode == Mode::User { warn!("USR mode has no SPSR."); return Err(GbaError::PrivilegedUserCode); }
        let new_mode = self.spsr[self.mode as u8 as usize].mode();
        self.change_mode(new_mode);
    }

    Ok(CpuAction::None)
}
*/
