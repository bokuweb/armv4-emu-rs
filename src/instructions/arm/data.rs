use std::cell::RefCell;
use std::rc::Rc;

use super::super::PipelineStatus;
use error::ArmError;

use super::shift::{ror, shift};
use bus::Bus;
use decoder::arm;
use types::*;

pub fn exec_mov<T>(
    bus: &Rc<RefCell<T>>,
    dec: &arm::Decoder,
    gpr: &mut [Word; 16],
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    if dec.has_I() {
        let src2 = dec.get_src2();
        gpr[dec.get_Rd()] = src2 as Word;
    } else {
        // TODO: implement later
    }
    Ok(PipelineStatus::Continue)
}

pub fn exec_and<T>(
    bus: &Rc<RefCell<T>>,
    dec: &arm::Decoder,
    gpr: &mut [Word; 16],
) -> Result<PipelineStatus, ArmError>
where
    T: Bus,
{
    let value = if dec.has_I() {
        ror(dec.get_imm8(), dec.get_rot() * 2)
    } else {
        let rm = dec.get_Rm() as usize;
        let sh = dec.get_sh();
        let shift_value = if dec.is_reg_offset() {
            dec.get_Rs()
        } else {
            dec.get_shamt5()
        };
        shift(sh, gpr[rm], shift_value)
    };
    gpr[dec.get_Rd()] = gpr[dec.get_Rn()] & value;
    Ok(PipelineStatus::Continue)
}
