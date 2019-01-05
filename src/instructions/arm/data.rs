use std::cell::RefCell;
use std::rc::Rc;

use super::super::PipelineStatus;
use error::ArmError;

use super::shift::{is_carry_over, lsl, ror, shift};
use bus::Bus;
use decoder::arm::Decoder;
use registers::psr::PSR;
use types::*;
use constants::*;

pub fn exec_data_processing<F>(
    gpr: &mut [Word; 16],
    dec: &Decoder,
    data_process: &mut F,
) -> Result<PipelineStatus, ArmError>
where
    F: FnMut(&mut [Word; 16], Word, Option<bool>),
{
    let (value, carry) = if dec.has_I() {
        let shift_value = dec.get_rot() * 2;
        (
            ror(dec.get_imm8(), shift_value),
            is_carry_over(dec.get_sh(), dec.get_imm8(), shift_value),
        )
    } else {
        let rm = dec.get_Rm() as usize;
        let shift_value = if dec.is_reg_offset() {
            dec.get_Rs()
        } else {
            dec.get_shamt5()
        };
        (
            shift(dec.get_sh(), gpr[rm], shift_value),
            is_carry_over(dec.get_sh(), gpr[rm], shift_value),
        )
    };
    data_process(gpr, value, carry);
    if dec.get_Rd() == PC {
        Ok(PipelineStatus::Flush)
    } else {
        Ok(PipelineStatus::Continue)
    }
}

pub fn exec_mov<T>(
    bus: &Rc<RefCell<T>>,
    dec: &Decoder,
    gpr: &mut [Word; 16],
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_data_processing(gpr, dec, &mut |gpr, value, _| {
        gpr[dec.get_Rd()] = value;
    })
}

pub fn exec_and<T>(
    bus: &Rc<RefCell<T>>,
    dec: &Decoder,
    gpr: &mut [Word; 16],
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_data_processing(gpr, dec, &mut |gpr, value, _| {
        gpr[dec.get_Rd()] = gpr[dec.get_Rn()] & value;
    })
}

pub fn exec_eor<T>(
    bus: &Rc<RefCell<T>>,
    dec: &Decoder,
    gpr: &mut [Word; 16],
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_data_processing(gpr, dec, &mut |gpr, value, _| {
        gpr[dec.get_Rd()] = gpr[dec.get_Rn()] ^ value;
    })
}

pub fn exec_sub<T>(
    bus: &Rc<RefCell<T>>,
    dec: &Decoder,
    gpr: &mut [Word; 16],
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_data_processing(gpr, dec, &mut |gpr, value, _| {
        gpr[dec.get_Rd()] = gpr[dec.get_Rn()].wrapping_sub(value);
    })
}

pub fn exec_rsb<T>(
    bus: &Rc<RefCell<T>>,
    dec: &Decoder,
    gpr: &mut [Word; 16],
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_data_processing(gpr, dec, &mut |gpr, value, _| {
        gpr[dec.get_Rd()] = value.wrapping_sub(gpr[dec.get_Rn()]);
    })
}

pub fn exec_add<T>(
    bus: &Rc<RefCell<T>>,
    dec: &Decoder,
    gpr: &mut [Word; 16],
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_data_processing(gpr, dec, &mut |gpr, value, _| {
        gpr[dec.get_Rd()] = gpr[dec.get_Rn()].wrapping_add(value);
    })
}

pub fn exec_adc<T>(
    bus: &Rc<RefCell<T>>,
    dec: &Decoder,
    gpr: &mut [Word; 16],
    cspr: &PSR,
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_data_processing(gpr, dec, &mut |gpr, value, _| {
        gpr[dec.get_Rd()] = gpr[dec.get_Rn()]
            .wrapping_add(value)
            .wrapping_add(if cspr.get_C() { 1 } else { 0 });
    })
}

pub fn exec_sbc<T>(
    bus: &Rc<RefCell<T>>,
    dec: &Decoder,
    gpr: &mut [Word; 16],
    cspr: &PSR,
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_data_processing(gpr, dec, &mut |gpr, value, _| {
        gpr[dec.get_Rd()] = gpr[dec.get_Rn()]
            .wrapping_sub(value)
            .wrapping_sub(if cspr.get_C() { 0 } else { 1 });
    })
}

pub fn exec_rsc<T>(
    bus: &Rc<RefCell<T>>,
    dec: &Decoder,
    gpr: &mut [Word; 16],
    cspr: &PSR,
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_data_processing(gpr, dec, &mut |gpr, value, _| {
        gpr[dec.get_Rd()] = gpr[dec.get_Rn()]
            .wrapping_sub(value)
            .wrapping_sub(if cspr.get_C() { 0 } else { 1 });
    })
}

pub fn exec_tst<T>(
    bus: &Rc<RefCell<T>>,
    dec: &Decoder,
    gpr: &mut [Word; 16],
    cspr: &mut PSR,
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_data_processing(gpr, dec, &mut |gpr, value, carry| {
        let tst = gpr[dec.get_Rn()] & value;
        cspr.set_N(tst >> 31 != 0);
        cspr.set_Z(tst == 0);
        if let Some(c) = carry {
            cspr.set_C(c);
        }
    })
}

pub fn exec_teq<T>(
    bus: &Rc<RefCell<T>>,
    dec: &Decoder,
    gpr: &mut [Word; 16],
    cspr: &mut PSR,
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_data_processing(gpr, dec, &mut |gpr, value, carry| {
        let teq = gpr[dec.get_Rn()] ^ value;
        cspr.set_N(teq >> 31 != 0);
        cspr.set_Z(teq == 0);
        if let Some(c) = carry {
            cspr.set_C(c);
        }
    })
}

pub fn exec_cmp<T>(
    bus: &Rc<RefCell<T>>,
    dec: &Decoder,
    gpr: &mut [Word; 16],
    cspr: &mut PSR,
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_data_processing(gpr, dec, &mut |gpr, value, _| {
        let rn = gpr[dec.get_Rn()];
        let cmp = rn.wrapping_sub(value);
        cspr.set_N(cmp >> 31 != 0);
        cspr.set_Z(cmp == 0);
        let (_, v) = (rn as i32).overflowing_sub(value as i32);
        cspr.set_V(v);
        // NOTE: Should we consider to shifted carry?
        cspr.set_C(rn >= value);
    })
}

pub fn exec_cmn<T>(
    bus: &Rc<RefCell<T>>,
    dec: &Decoder,
    gpr: &mut [Word; 16],
    cspr: &mut PSR,
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_data_processing(gpr, dec, &mut |gpr, value, _| {
        let rn = gpr[dec.get_Rn()];
        let cmn = (rn as u64).wrapping_add(value as u64);
        cspr.set_N((cmn as i32) < 0);
        cspr.set_Z(cmn == 0);
        let (_, v) = (rn as i32).overflowing_add(value as i32);
        cspr.set_V(v);
        cspr.set_C(cmn & (1 << 32) != 0);
    })
}

pub fn exec_orr<T>(
    bus: &Rc<RefCell<T>>,
    dec: &Decoder,
    gpr: &mut [Word; 16],
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_data_processing(gpr, dec, &mut |gpr, value, _| {
        gpr[dec.get_Rd()] = gpr[dec.get_Rn()] | value;
    })
}

pub fn exec_shift<T>(
    bus: &Rc<RefCell<T>>,
    dec: &Decoder,
    gpr: &mut [Word; 16],
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_data_processing(gpr, dec, &mut |gpr, value, _| {
        gpr[dec.get_Rd()] = value;
    })
}

pub fn exec_bic<T>(
    bus: &Rc<RefCell<T>>,
    dec: &Decoder,
    gpr: &mut [Word; 16],
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_data_processing(gpr, dec, &mut |gpr, value, _| {
        gpr[dec.get_Rd()] = gpr[dec.get_Rn()] & !value
    })
}

pub fn exec_mvn<T>(
    bus: &Rc<RefCell<T>>,
    dec: &Decoder,
    gpr: &mut [Word; 16],
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_data_processing(gpr, dec, &mut |gpr, value, _| gpr[dec.get_Rd()] = !value)
}

pub fn exec_rrx<T>(
    bus: &Rc<RefCell<T>>,
    dec: &Decoder,
    gpr: &mut [Word; 16],
    cspr: &PSR,
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    exec_data_processing(gpr, dec, &mut |gpr, value, _| {
        gpr[dec.get_Rd()] = value >> 1 | (if cspr.get_C() { 0x8000_0000 } else { 0 })
    })
}
