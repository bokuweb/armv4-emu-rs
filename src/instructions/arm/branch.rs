use super::super::PipelineStatus;
use constants::*;
use decoder::arm;
use error::ArmError;
use types::*;

pub fn exec_bl(dec: &arm::BaseDecoder, gpr: &mut [Word; 16]) -> Result<PipelineStatus, ArmError> {
    gpr[LR] = gpr[PC] - 4;
    exec_b(dec, gpr)
}

pub fn exec_b(dec: &arm::BaseDecoder, gpr: &mut [Word; 16]) -> Result<PipelineStatus, ArmError> {
    let imm = dec.get_imm24() as u32;
    let imm = (if imm & 0x0080_0000 != 0 {
        imm | 0xFF00_0000
    } else {
        imm
    }) as i32;
    gpr[PC] = (gpr[PC] as i32 + imm * 4) as Word;
    Ok(PipelineStatus::Flush)
}
