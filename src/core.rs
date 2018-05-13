use std::cell::RefCell;
use std::error;
use std::fmt;
use std::rc::Rc;
use std::result::Result::Err;

use instructions::{arm, shift};
use types::*;

pub const INITIAL_PIPELINE_WAIT: u8 = 2;
pub const PC_OFFSET: usize = 2;

#[derive(Debug, PartialEq, Clone)]
pub enum ArmError {
    UnknownError,
}

impl error::Error for ArmError {
    fn description(&self) -> &str {
        match *self {
            ArmError::UnknownError => "Unknown ARM error",
        }
    }
}

impl fmt::Display for ArmError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ArmError::UnknownError => write!(f, "Unknown ARM error"),
        }
    }
}

enum Arm {
    NOP,
    NOP_RAW,
}

//enum PipelineState {
//    WAIT,
//    Ready,
//}

enum CpuMode {
    System,
    Supervisor,
    FIQ,
}

#[derive(Debug)]
enum PipelineStatus {
    Flush,
    Continue,
}

#[derive(Debug, PartialEq)]
enum CpuState {
    ARM,
    Thumb,
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct PSR;

impl PSR {
    pub fn default() -> Self {
        PSR
    }
}

pub const SP: usize = 13;
pub const LR: usize = 14;
pub const PC: usize = 15;

struct CpuBus;

pub struct ARMv4<T>
where
    T: Bus,
{
    pub gpr: [u32; 16],
    bus: Rc<RefCell<T>>,
    pipeline_wait: u8,
    cpsr: PSR,
    spsr: [PSR; 7],
    mode: CpuMode,
    state: CpuState,
    irq_disable: bool,
    fiq_disable: bool,
    optimise_swi: bool,
}

type Word = u32;
type HalfWord = u16;

pub trait Bus {
    fn read_byte(&self, addr: u32) -> Byte;
    fn read_word(&self, addr: u32) -> Word;
}

impl<T> ARMv4<T>
where
    T: Bus,
{
    pub fn new(bus: Rc<RefCell<T>>) -> ARMv4<T>
    where
        T: Bus,
    {
        ARMv4 {
            bus: bus,
            pipeline_wait: INITIAL_PIPELINE_WAIT,

            gpr: [0; 16],
            cpsr: PSR::default(),
            spsr: [PSR::default(); 7],
            mode: CpuMode::System,
            state: CpuState::ARM,
            irq_disable: false,
            fiq_disable: false,
            optimise_swi: false,
        }
    }

    pub fn reset(&mut self) {
        self.gpr[PC] = 0x00000000;

        self.cpsr = PSR::default();

        self.mode = CpuMode::Supervisor;
        self.state = CpuState::ARM;
        self.irq_disable = true;
        self.fiq_disable = true;
    }

    fn flush_pipeline(&mut self) {
        self.pipeline_wait = INITIAL_PIPELINE_WAIT;
    }

    fn increment_pc(&mut self) {
        let next = if self.state == CpuState::ARM { 4 } else { 2 };
        self.gpr[PC] = self.gpr[PC].wrapping_add(next);
    }

    fn execute(&mut self, inst: arm::Instruction) -> Result<(), ArmError> {
        debug!("decoded instruction = {:?}", inst);
        let pipeline_status = match inst.opcode {
            arm::Opcode::LDR => self.exec_ldr(inst)?,
            arm::Opcode::STR => self.exec_str(inst)?,
            arm::Opcode::LDRB => self.exec_ldrb(inst)?,
            arm::Opcode::STRB => unimplemented!(),
            arm::Opcode::MOV => self.exec_mov(inst)?,
            arm::Opcode::B => self.exec_b(inst)?,
            arm::Opcode::BL => self.exec_bl(inst)?,
            //arm::Opcode::Undefined => unimplemented!(),
            //arm::Opcode::NOP => unimplemented!(),
            //// arm::Opcode::SWI => unimplemented!(),
            // ArmOpcode::Unknown => self.execute_unknown(inst),
            _ => unimplemented!(),
        };

        match pipeline_status {
            PipelineStatus::Continue => self.increment_pc(),
            PipelineStatus::Flush => self.flush_pipeline(),
        };
        Ok(())
    }

    #[allow(non_snake_case)]
    fn exec_ldrb(&mut self, inst: arm::Instruction) -> Result<PipelineStatus, ArmError> {
        let mut base = self.gpr[inst.get_Rn()];
        // INFO: Treat as imm12 if not I.
        if !inst.has_I() {
            let src2 = inst.get_src2() as i32;
            let offset = (src2 * if inst.is_plus_offset() { 1 } else { -1 }) as i32;
            base = (base as i32 + offset) as Word;
        } else {
            debug!(" TODO: implement later");
            unimplemented!();
        }
        let Rd = inst.get_Rd();
        self.gpr[Rd] = self.bus.borrow().read_byte(base) as Word;
        if Rd == PC {
            Ok(PipelineStatus::Flush)
        } else {
            Ok(PipelineStatus::Continue)
        }
    }

    #[allow(non_snake_case)]
    fn exec_ldr(&mut self, inst: arm::Instruction) -> Result<PipelineStatus, ArmError> {
        let mut base = self.gpr[inst.get_Rn()];
        // INFO: Treat as imm12 if not I.
        let offset = if !inst.has_I() {
            inst.get_src2() as u32
        } else {
            let rm = inst.get_Rm() as usize;
            let sh = inst.get_sh();
            let shamt5 = inst.get_shamt5();
            match sh {
                arm::Shift::LSL => shift::lsl(self.gpr[rm], shamt5),
                arm::Shift::LSR => shift::lsr(self.gpr[rm], shamt5),
                arm::Shift::ASR => shift::asr(self.gpr[rm], shamt5),
                arm::Shift::ROR => shift::ror(self.gpr[rm], shamt5),
            }
        };
        let offset_base = if inst.is_plus_offset() {
            (base + offset) as Word
        } else {
            (base - offset) as Word
        };
        if inst.is_pre_indexed() {
            base = offset_base;
        }
        let Rd = inst.get_Rd();
        self.gpr[Rd] = self.bus.borrow().read_word(base);
        if !inst.is_pre_indexed() {
            self.gpr[inst.get_Rn()] = offset_base;
        } else if inst.is_write_back() {
            self.gpr[inst.get_Rn()] = base;
        }
        if Rd == PC {
            Ok(PipelineStatus::Flush)
        } else {
            Ok(PipelineStatus::Continue)
        }
    }

    fn exec_str(&mut self, inst: arm::Instruction) -> Result<PipelineStatus, ArmError> {
        Ok(PipelineStatus::Continue)
    }

    fn exec_mov(&mut self, inst: arm::Instruction) -> Result<PipelineStatus, ArmError> {
        if inst.has_I() {
            let src2 = inst.get_src2();
            self.gpr[inst.get_Rd()] = src2 as Word;
        } else {
            // TODO: implement later
        }
        Ok(PipelineStatus::Continue)
    }

    fn exec_bl(&mut self, inst: arm::Instruction) -> Result<PipelineStatus, ArmError> {
        self.gpr[LR] = self.gpr[PC] - 4;
        self.exec_b(inst)
    }

    fn exec_b(&mut self, inst: arm::Instruction) -> Result<PipelineStatus, ArmError> {
        let imm = inst.get_imm24() as u32;
        let imm = (if imm & 0x0080_0000 != 0 {
            imm | 0xFF00_0000
        } else {
            imm
        }) as i32;
        self.gpr[PC] = (self.gpr[PC] as i32 + imm * 4) as Word;
        Ok(PipelineStatus::Flush)
    }

    pub fn tick(&mut self) -> Result<(), ArmError> {
        if self.pipeline_wait > 0 {
            self.pipeline_wait -= 1;
            self.increment_pc();
            return Ok(());
        }
        debug!("registers = {:?}", self.gpr);
        match self.state {
            CpuState::ARM => {
                let fetched = self.bus
                    .borrow()
                    .read_word(self.gpr[PC] - (PC_OFFSET * 4) as u32);
                debug!("fetched code = {:x}", fetched);
                let decoded = arm::Instruction::decode(fetched);
                self.execute(decoded)
            }
            // TODO: Thumb mode
            _ => unimplemented!(),
        }
    }

    pub fn get_gpr(&self, n: usize) -> Word {
        self.gpr[n]
    }

    pub fn set_gpr(&mut self, n: usize, data: u32) {
        self.gpr[n] = data;
    }
}

#[cfg(test)]
mod test {
    extern crate byteorder;
    extern crate env_logger;

    use super::*;
    use byteorder::{ByteOrder, LittleEndian};
    use std::cell::RefCell;
    use std::rc::Rc;

    trait CpuTest {
        fn run_immediately(&mut self);
    }

    struct MockBus {
        pub mem: Vec<u8>,
    }

    impl MockBus {
        pub fn new() -> Self {
            MockBus { mem: vec![0; 1024] }
        }

        pub fn set(&mut self, addr: Word, data: Word) {
            LittleEndian::write_u32(&mut self.mem[(addr as usize)..], data);
        }
    }

    impl Bus for MockBus {
        fn read_byte(&self, addr: Word) -> Byte {
            self.mem[addr as usize]
        }

        fn read_word(&self, addr: Word) -> Word {
            LittleEndian::read_u32(&self.mem[(addr as usize)..])
        }
    }

    impl CpuTest for ARMv4<MockBus> {
        fn run_immediately(&mut self) {
            for _ in 0..(INITIAL_PIPELINE_WAIT + 1) {
                self.tick();
            }
        }
    }

    fn setup() {
        use std::sync::{Once, ONCE_INIT};
        static INIT: Once = ONCE_INIT;
        INIT.call_once(|| env_logger::init());
    }

    #[test]
    // tick
    fn increment_pc_by_tick() {
        setup();
        let mut bus = MockBus::new();
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.tick();
        assert_eq!(arm.get_gpr(PC), 0x0000_0004);
    }

    #[test]
    // ldr pc, =0x8000_0000
    fn ldr_pc_eq0x8000_0000() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE51F_F004);
        &bus.set(0x4, 0x8000_0000);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.run_immediately();
        assert_eq!(arm.get_gpr(PC), 0x8000_0000);
    }

    #[test]
    // LDR offset addressing
    // ldrb r1, [r0]
    fn ldrb_r1_r0() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE5D0_1000);
        &bus.set(0x100, 0xAAAA_5555);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(0, 0x100);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(1), 0x55);
        assert_eq!(arm.get_gpr(0), 0x0000_0100);
    }

    // LDR post index addressing
    // ldr	r0, [r1], #4
    #[test]
    fn ldrb_r0_r1_4() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE491_0004);
        &bus.set(0x100, 0xAAAA_5555);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(1, 0x100);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(0), 0xAAAA_5555);
        assert_eq!(arm.get_gpr(1), 0x0104);
    }

    #[test]
    // ldr r8, [r9, r2, lsl #2]
    // R8 <- mem[r9 + (r2 << 2)]
    fn ldr_r8_r9_r2_lsl_2() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE799_8102);
        &bus.set(0x140, 0xAA55_55AA);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(2, 0x10);
        arm.set_gpr(9, 0x100);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(8), 0xAA55_55AA);
    }

    #[test]
    // mov r0, #1
    fn mov_r0_imm1() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE3A0_0001);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.run_immediately();
        assert_eq!(arm.get_gpr(0), 0x0000_0001);
    }

    #[test]
    // b pc-2
    fn b_pc_sub_2() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0000_0000, 0xEAFF_FFFE);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.run_immediately();
        assert_eq!(arm.get_gpr(PC), 0x0000_0000);
    }

    #[test]
    // bl pc-2
    fn bl_pc_sub_2() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0000_0000, 0xEBFF_FFFE);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.run_immediately();
        assert_eq!(arm.get_gpr(PC), 0x0000_0000);
        assert_eq!(arm.get_gpr(LR), 0x0000_0004);
    }
}
