use std::cell::RefCell;
use std::rc::Rc;

use error::ArmError;
use super::super::PipelineStatus;

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
