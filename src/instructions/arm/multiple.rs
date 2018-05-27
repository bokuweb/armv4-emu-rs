use std::cell::RefCell;
use std::rc::Rc;

use super::super::PipelineStatus;
use error::ArmError;

use super::data::*;
use super::shift::{is_carry_over, lsl, ror, shift};
use bus::Bus;
use constants::*;
use decoder::arm;
use registers::psr::PSR;
use types::*;

pub fn exec_multiple<F>(
    gpr: &mut [Word; 16],
    dec: &arm::BaseDecoder,
    multiple: &mut F,
) -> Result<PipelineStatus, ArmError>
where
    F: FnMut(&mut [Word; 16]),
{
    multiple(gpr);
    if dec.get_Rd() == PC {
        Ok(PipelineStatus::Flush)
    } else {
        Ok(PipelineStatus::Continue)
    }
}

pub fn exec_mul<T>(
    bus: &Rc<RefCell<T>>,
    dec: &arm::BaseDecoder,
    gpr: &mut [Word; 16],
    cspr: &PSR,
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_multiple(gpr, dec, &mut |gpr| {
        gpr[dec.get_Rd()] = ((gpr[dec.get_Rn()] as u64) * gpr[dec.get_Rm()] as u64) as u32;
    })
}

pub fn exec_mla<T>(
    bus: &Rc<RefCell<T>>,
    dec: &arm::BaseDecoder,
    gpr: &mut [Word; 16],
    cspr: &PSR,
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_multiple(gpr, dec, &mut |gpr| {
        gpr[dec.get_Rd()] = (((gpr[dec.get_Rn()] as u64) * gpr[dec.get_Rm()] as u64)
            + gpr[dec.get_Ra()] as u64) as u32;
    })
}

pub fn exec_umull<T>(
    bus: &Rc<RefCell<T>>,
    dec: &arm::BaseDecoder,
    gpr: &mut [Word; 16],
    cspr: &PSR,
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_multiple(gpr, dec, &mut |gpr| {
        let mul = (gpr[dec.get_Rn()] as u64) * gpr[dec.get_Rm()] as u64;
        gpr[dec.get_Ra()] = mul as u32;
        gpr[dec.get_Rd()] = (mul >> 32) as u32;
    })
}

pub fn exec_umlal<T>(
    bus: &Rc<RefCell<T>>,
    dec: &arm::BaseDecoder,
    gpr: &mut [Word; 16],
    cspr: &PSR,
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_multiple(gpr, dec, &mut |gpr| {
        let mul = (gpr[dec.get_Rn()] as u64) * gpr[dec.get_Rm()] as u64;
        let base = ((gpr[dec.get_Rd()] as u64) << 32) + (gpr[dec.get_Ra()] as u64);
        let result = mul + base;
        gpr[dec.get_Ra()] = result as u32;
        gpr[dec.get_Rd()] = (result >> 32) as u32;
    })
}

pub fn exec_smull<T>(
    bus: &Rc<RefCell<T>>,
    dec: &arm::BaseDecoder,
    gpr: &mut [Word; 16],
    cspr: &PSR,
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_multiple(gpr, dec, &mut |gpr| {
        let mul = (gpr[dec.get_Rn()] as i32 as i64) * gpr[dec.get_Rm()] as i32 as i64;
        gpr[dec.get_Ra()] = mul as u32;
        gpr[dec.get_Rd()] = (mul >> 32) as u32;
    })
}

pub fn exec_smlal<T>(
    bus: &Rc<RefCell<T>>,
    dec: &arm::BaseDecoder,
    gpr: &mut [Word; 16],
    cspr: &PSR,
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_multiple(gpr, dec, &mut |gpr| {
        let mul = (gpr[dec.get_Rn()] as i32 as i64) * gpr[dec.get_Rm()] as i32 as i64;
        let base = (((gpr[dec.get_Rd()] as u64) << 32) + (gpr[dec.get_Ra()] as u64)) as i64;
        let result = mul + base;
        gpr[dec.get_Ra()] = result as u32;
        gpr[dec.get_Rd()] = (result >> 32) as u32;
    })
}
