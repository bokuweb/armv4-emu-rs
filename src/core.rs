use std::cell::RefCell;
use std::error;
use std::fmt;
use std::rc::Rc;
use std::result::Result::Err;

use bus::Bus;
use constants::*;
use decoder::arm;
use error::ArmError;
use registers::psr::PSR;
use instructions::arm::branch::*;
use instructions::arm::data::*;
use instructions::arm::memory::*;
use instructions::PipelineStatus;
use types::*;

pub const INITIAL_PIPELINE_WAIT: u8 = 2;
pub const PC_OFFSET: usize = 2;

enum Arm {
    NOP,
    NOP_RAW,
}

enum CpuMode {
    System,
    Supervisor,
    FIQ,
}

#[derive(Debug, PartialEq)]
enum CpuState {
    ARM,
    Thumb,
}

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

    fn execute(&mut self, dec: arm::Decoder) -> Result<(), ArmError> {
        debug!("decoded instruction = {:?}", dec);
        let pipeline_status = {
            match dec.opcode {
                arm::Opcode::LDR => exec_ldr(&self.bus, &dec, &mut self.gpr)?,
                arm::Opcode::STR => exec_str(&self.bus, &dec, &mut self.gpr)?,
                arm::Opcode::LDRB => exec_ldrb(&self.bus, &dec, &mut self.gpr)?,
                arm::Opcode::STRB => exec_strb(&self.bus, &dec, &mut self.gpr)?,
                arm::Opcode::AND => exec_and(&self.bus, &dec, &mut self.gpr)?,
                arm::Opcode::EOR => exec_eor(&self.bus, &dec, &mut self.gpr)?,
                arm::Opcode::SUB => exec_sub(&self.bus, &dec, &mut self.gpr)?,
                arm::Opcode::RSB => exec_rsb(&self.bus, &dec, &mut self.gpr)?,
                arm::Opcode::ADD => exec_add(&self.bus, &dec, &mut self.gpr)?,
                arm::Opcode::MOV => exec_mov(&self.bus, &dec, &mut self.gpr)?,
                arm::Opcode::B => exec_b(&dec, &mut self.gpr)?,
                arm::Opcode::BL => exec_bl(&dec, &mut self.gpr)?,
                //arm::Opcode::Undefined => unimplemented!(),
                //arm::Opcode::NOP => unimplemented!(),
                //// arm::Opcode::SWI => unimplemented!(),
                // ArmOpcode::Unknown => self.execute_unknown(dec),
                _ => unimplemented!(),
            }
        };
        match pipeline_status {
            PipelineStatus::Continue => self.increment_pc(),
            PipelineStatus::Flush => self.flush_pipeline(),
        };
        Ok(())
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
                let decoded = arm::Decoder::decode(fetched);
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
    use memory::readable::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    trait CpuTest {
        fn run_immediately(&mut self);
        fn get_mem(&self, addr: usize) -> u32;
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

        fn write_byte(&mut self, addr: Word, data: u8) {
            self.mem[(addr as usize)] = data;
        }

        fn write_word(&mut self, addr: Word, data: u32) {
            LittleEndian::write_u32(&mut self.mem[(addr as usize)..], data);
        }
    }

    impl CpuTest for ARMv4<MockBus> {
        fn run_immediately(&mut self) {
            for _ in 0..(INITIAL_PIPELINE_WAIT + 1) {
                self.tick();
            }
        }

        fn get_mem(&self, addr: usize) -> u32 {
            LittleEndian::read_u32(&self.bus.borrow().mem[(addr as usize)..])
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

    #[test]
    // LDR post index addressing
    // ldr	r0, [r1], #4
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
    // 	str r4, [r3]
    fn str_r4_r3() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE583_4000);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(3, 0x200);
        arm.set_gpr(4, 0xAA55_55AA);
        arm.run_immediately();
        assert_eq!(arm.get_mem(0x200), 0xAA55_55AA);
    }

    #[test]
    // 	strb r4, [r3]
    fn strb_r4_r3() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE5C3_4000);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(3, 0x200);
        arm.set_gpr(4, 0x1155_55AA);
        arm.run_immediately();
        assert_eq!(arm.get_mem(0x200), 0x0000_00AA);
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
    // and r3, r1, r2
    // r3 <- r1 & r2
    fn and_r3_r1_r2() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE001_3002);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(1, 0xAA55_55AA);
        arm.set_gpr(2, 0xA050_1122);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(3), 0xA050_1122);
    }

    #[test]
    // eor r3, r1, r2
    // r3 <- r1 ^ r2
    fn eor_r3_r1_r2() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE021_3002);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(1, 0xAA55_55AA);
        arm.set_gpr(2, 0xA050_1122);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(3), 0x0A05_4488);
    }    

    #[test]
    // sub r3, r1, r2
    // r3 <- r1 - r2
    fn sub_r3_r1_r2() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE041_3002);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(1, 0xAA55_5588);
        arm.set_gpr(2, 0xA050_1122);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(3), 0x0A05_4466);
    }    

    #[test]
    // rsb r3, r1, r2
    // r3 <- r2 - r1
    fn rsb_r3_r1_r2() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE061_3002);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(1, 0x1234_5678);
        arm.set_gpr(2, 0x2345_6789);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(3), 0x1111_1111);
    }        

    #[test]
    // add r3, r1, r2
    // r3 <- r1 + r2
    fn add_r3_r1_r2() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE081_3002);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(1, 0x1234_5678);
        arm.set_gpr(2, 0x2345_6789);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(3), 0x3579_BE01);
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
