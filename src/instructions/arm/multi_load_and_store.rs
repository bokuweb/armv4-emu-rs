use std::cell::RefCell;
use std::rc::Rc;

use bus::Bus;
use constants::*;
use decoder::arm;
use types::*;

use super::super::PipelineStatus;
use super::shift::shift;
use error::ArmError;

//!     COND 01I+  -BWL RegN  RegD offs  offs offs | LDR/STR
//!     COND 000+  -0WL RegN  RegD 0000  1xx1 RegM | LDRH/STRH/LDRSB/LDRSH depending on Op(xx)
//!     COND 000+  -1WL RegN  RegD imm_  1xx1 imm_ | LDRH/STRH/LDRSB/LDRSH depending on Op(xx) with Offset=imm_imm_
//!     COND 100+  -RWL RegN  regs regs  regs regs | LDM/STM with register list regsregsregsregs
//! Bit Flags:
//!     I: 1=shftIsRegister,  0=shftIsImmediate
//!     F: 1=BranchWithLink,  0=BranchWithoutLink
//!     +: 1=PreIndexing,     0=PostIndexing
//!     -: 1=AddOffset,       0=SubtractOffset
//!     P: 1=SPSR,            0=CPSR
//!     U: 1=Signed,          0=Unsigned
//!     B: 1=TransferByte,    0=TransferWord
//!     R: 1=ForceUserMode,   0=NoForceUserMode
//!     N: 1=TransferAllRegs, 0=TransferSingleReg
//!     A: 1=Accumulate,      0=DoNotAccumulate
//!     W: 1=AutoIncrement,   0=NoWriteBack
//!     S: 1=SetFlags,        0=DoNotSetFlags
//!     L: 1=Load,            0=Store
//! 
fn exec_multi_memory_processing<F>(
    gpr: &mut [u32; 16],
    dec: &arm::Decoder,
    load_or_store: F,
) -> Result<PipelineStatus, ArmError>
where
    F: Fn(&mut [u32; 16], u32),
{
    let mut base = gpr[dec.get_Rn()];
    // INFO: Treat as imm12 if not I.
    // let offset = if !dec.has_I() {
    //     dec.get_src2() as u32
    // } else {
    //     let rm = dec.get_Rm() as usize;
    //     let sh = dec.get_sh();
    //     let shamt5 = dec.get_shamt5();
    //     shift(sh, gpr[rm], shamt5)
    // };
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
    if dec.get_Rd() == PC {
        Ok(PipelineStatus::Flush)
    } else {
        Ok(PipelineStatus::Continue)
    }
}

#[allow(non_snake_case)]
pub fn exec_ldr<T>(
    bus: &Rc<RefCell<T>>,
    dec: &arm::Decoder,
    gpr: &mut [Word; 16],
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_memory_processing(gpr, dec, |gpr, base| {
        gpr[dec.get_Rd()] = bus.borrow().read_word(base);
    })
}
