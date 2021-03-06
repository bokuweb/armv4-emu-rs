use std::cell::RefCell;
use std::rc::Rc;

use bus::Bus;
use constants::*;
use decoder::arm;
use error::ArmError;
use instructions::arm::branch::*;
use instructions::arm::data::*;
use instructions::arm::extra_memory::*;
use instructions::arm::memory::*;
use instructions::arm::multi_load_and_store::*;
use instructions::arm::multiple::*;
use instructions::PipelineStatus;
use registers::psr::PSR;
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
            bus,
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

    fn execute(&mut self, dec: &arm::Decoder) -> Result<(), ArmError> {
        debug!("execute {:?}", dec.opcode());
        let pipeline_status = {
            match dec.opcode() {
                arm::Opcode::AND => exec_and(&self.bus, dec, &mut self.gpr)?,
                arm::Opcode::EOR => exec_eor(&self.bus, dec, &mut self.gpr)?,
                arm::Opcode::SUB => exec_sub(&self.bus, dec, &mut self.gpr)?,
                arm::Opcode::RSB => exec_rsb(&self.bus, dec, &mut self.gpr)?,
                arm::Opcode::ADD => exec_add(&self.bus, dec, &mut self.gpr)?,
                arm::Opcode::ADC => exec_adc(&self.bus, dec, &mut self.gpr, &self.cpsr)?,
                arm::Opcode::SBC => exec_sbc(&self.bus, dec, &mut self.gpr, &self.cpsr)?,
                arm::Opcode::RSC => exec_rsc(&self.bus, dec, &mut self.gpr, &self.cpsr)?,
                arm::Opcode::TST => exec_tst(&self.bus, dec, &mut self.gpr, &mut self.cpsr)?,
                arm::Opcode::TEQ => exec_teq(&self.bus, dec, &mut self.gpr, &mut self.cpsr)?,
                arm::Opcode::CMP => exec_cmp(&self.bus, dec, &mut self.gpr, &mut self.cpsr)?,
                arm::Opcode::CMN => exec_cmn(&self.bus, dec, &mut self.gpr, &mut self.cpsr)?,
                arm::Opcode::ORR => exec_orr(&self.bus, dec, &mut self.gpr)?,
                arm::Opcode::MOV => exec_mov(&self.bus, dec, &mut self.gpr)?,
                arm::Opcode::LSL => exec_shift(&self.bus, dec, &mut self.gpr)?,
                arm::Opcode::LSR => exec_shift(&self.bus, dec, &mut self.gpr)?,
                arm::Opcode::ASR => exec_shift(&self.bus, dec, &mut self.gpr)?,
                arm::Opcode::RRX => exec_rrx(&self.bus, dec, &mut self.gpr, &self.cpsr)?,
                arm::Opcode::ROR => exec_shift(&self.bus, dec, &mut self.gpr)?,
                arm::Opcode::BIC => exec_bic(&self.bus, dec, &mut self.gpr)?,
                arm::Opcode::MVN => exec_mvn(&self.bus, dec, &mut self.gpr)?,
                arm::Opcode::MUL => exec_mul(&self.bus, dec, &mut self.gpr, &self.cpsr)?,
                arm::Opcode::MLA => exec_mla(&self.bus, dec, &mut self.gpr, &self.cpsr)?,
                arm::Opcode::UMULL => exec_umull(&self.bus, dec, &mut self.gpr, &self.cpsr)?,
                arm::Opcode::UMLAL => exec_umlal(&self.bus, dec, &mut self.gpr, &self.cpsr)?,
                arm::Opcode::SMULL => exec_smull(&self.bus, dec, &mut self.gpr, &self.cpsr)?,
                arm::Opcode::SMLAL => exec_smlal(&self.bus, dec, &mut self.gpr, &self.cpsr)?,
                arm::Opcode::LDR => exec_ldr(&self.bus, dec, &mut self.gpr)?,
                arm::Opcode::STR => exec_str(&self.bus, dec, &mut self.gpr)?,
                arm::Opcode::LDRB => exec_ldrb(&self.bus, dec, &mut self.gpr)?,
                arm::Opcode::STRB => exec_strb(&self.bus, dec, &mut self.gpr)?,
                arm::Opcode::STRH => exec_strh(&self.bus, dec, &mut self.gpr)?,
                arm::Opcode::LDRH => exec_ldrh(&self.bus, dec, &mut self.gpr)?,
                arm::Opcode::LDRSB => exec_ldrsb(&self.bus, dec, &mut self.gpr)?,
                arm::Opcode::LDRSH => exec_ldrsh(&self.bus, dec, &mut self.gpr)?,
                arm::Opcode::B => exec_b(dec, &mut self.gpr)?,
                arm::Opcode::BL => exec_bl(dec, &mut self.gpr)?,
                arm::Opcode::LDM => exec_ldm(&self.bus, dec, &mut self.gpr)?,
                arm::Opcode::STM => exec_stm(&self.bus, dec, &mut self.gpr)?,
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
                debug!("fetch addr = 0x{:x}", self.gpr[PC] - (PC_OFFSET * 4) as u32);
                let fetched = self
                    .bus
                    .borrow()
                    .read_word(self.gpr[PC] - (PC_OFFSET * 4) as u32);
                debug!("fetched code = {:x}", fetched);
                let decoder = &*arm::decode(fetched);
                self.execute(decoder)
            }
            // TODO: Thumb mode
            _ => unimplemented!(),
        }
    }

    pub fn get_gpr(&self, n: usize) -> Word {
        self.gpr[n]
    }

    pub fn get_cpsr(&self) -> PSR {
        self.cpsr
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
    // adc r3, r1, r2
    // r3 <- r1 + r2 + C
    fn adc_r3_r1_r2() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE0A1_3002);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.cpsr.set_C(true);
        arm.set_gpr(1, 0x1234_5678);
        arm.set_gpr(2, 0x2345_6789);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(3), 0x3579_BE02);
    }

    #[test]
    // sbc r3, r1, r2
    // r3 <- r1 - r2 - !C
    fn sbc_r3_r1_r2_with_set_c() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE0E1_3002);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.cpsr.set_C(true);
        arm.set_gpr(1, 0x2345_6789);
        arm.set_gpr(2, 0x1234_5678);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(3), 0x1111_1111);
    }

    #[test]
    // sbc r3, r1, r2
    // r3 <- r1 - r2 - !C
    fn sbc_r3_r1_r2_with_cleared_c() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE0E1_3002);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.cpsr.set_C(false);
        arm.set_gpr(1, 0x2345_6789);
        arm.set_gpr(2, 0x1234_5678);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(3), 0x1111_1110);
    }

    #[test]
    // rsc r3, r1, r2
    // r3 <- r2 - r1 -!C
    fn rsc_r3_r1_r2() {
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
    // tst r0, r1
    fn tst_r0_r1() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE110_0001);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(0, 0x8234_5678);
        arm.set_gpr(1, 0x8345_6789);
        arm.run_immediately();
        assert_eq!(arm.get_cpsr().get_C(), false);
        assert_eq!(arm.get_cpsr().get_N(), true);
        assert_eq!(arm.get_cpsr().get_Z(), false);
    }

    #[test]
    // tst r1, r2, asr #4
    fn tst_r1_r2_asr_4_without_zero() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE111_0242);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(1, 0x8234_5678);
        arm.set_gpr(2, 0x80FF_0008);
        arm.run_immediately();
        assert_eq!(arm.get_cpsr().get_C(), true);
        assert_eq!(arm.get_cpsr().get_N(), true);
        assert_eq!(arm.get_cpsr().get_Z(), false);
    }

    #[test]
    // tst r1, r2, asr #4
    fn tst_r1_r2_asr_4_with_zero() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE111_0242);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(1, 0x8234_5678);
        arm.set_gpr(2, 0x0000_0000);
        arm.run_immediately();
        assert_eq!(arm.get_cpsr().get_C(), false);
        assert_eq!(arm.get_cpsr().get_N(), false);
        assert_eq!(arm.get_cpsr().get_Z(), true);
    }

    #[test]
    // teq r1, r2
    fn tst_r1_r2_equal() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE131_0002);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(1, 0x8234_5678);
        arm.set_gpr(2, 0x8234_5678);
        arm.run_immediately();
        assert_eq!(arm.get_cpsr().get_C(), false);
        assert_eq!(arm.get_cpsr().get_N(), false);
        assert_eq!(arm.get_cpsr().get_Z(), true);
    }

    #[test]
    // teq r1, r2
    fn tst_r1_r2_not_equal() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE131_0002);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(1, 0x8234_5678);
        arm.set_gpr(2, 0x0234_5678);
        arm.run_immediately();
        assert_eq!(arm.get_cpsr().get_C(), false);
        assert_eq!(arm.get_cpsr().get_N(), true);
        assert_eq!(arm.get_cpsr().get_Z(), false);
    }

    #[test]
    // cmp r1, r2
    fn cmp_r1_r2_carry() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE151_0002);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(1, 0x0000_0002);
        arm.set_gpr(2, 0x0000_0001);
        arm.run_immediately();
        assert_eq!(arm.get_cpsr().get_C(), true);
        assert_eq!(arm.get_cpsr().get_N(), false);
        assert_eq!(arm.get_cpsr().get_Z(), false);
        assert_eq!(arm.get_cpsr().get_V(), false);
    }

    #[test]
    // cmp r1, r2
    fn cmp_r1_r2_without_carry() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE151_0002);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(1, 0x0000_0001);
        arm.set_gpr(2, 0x0000_0002);
        arm.run_immediately();
        assert_eq!(arm.get_cpsr().get_C(), false);
        assert_eq!(arm.get_cpsr().get_N(), true);
        assert_eq!(arm.get_cpsr().get_Z(), false);
        assert_eq!(arm.get_cpsr().get_V(), false);
    }

    #[test]
    // cmp r1, r2
    fn cmp_r1_r2_with_overflow() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE151_0002);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(1, 0x8000_0000);
        arm.set_gpr(2, 0x0000_0001);
        arm.run_immediately();
        assert_eq!(arm.get_cpsr().get_C(), true);
        assert_eq!(arm.get_cpsr().get_N(), false);
        assert_eq!(arm.get_cpsr().get_Z(), false);
        assert_eq!(arm.get_cpsr().get_V(), true);
    }

    #[test]
    // cmn r1, r2
    fn cmn_r1_r2_with_overflow() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE171_0001);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(1, 0x7FFF_FFFF);
        arm.set_gpr(2, 0x0000_0001);
        arm.run_immediately();
        assert_eq!(arm.get_cpsr().get_C(), false);
        assert_eq!(arm.get_cpsr().get_N(), true);
        assert_eq!(arm.get_cpsr().get_Z(), false);
        assert_eq!(arm.get_cpsr().get_V(), true);
    }

    #[test]
    // orr r1, r2, r3
    fn orr_r1_r2_r3() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE182_1003);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(2, 0xAA55_55AA);
        arm.set_gpr(3, 0x5500_AA00);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(1), 0xFF55_FFAA);
    }

    #[test]
    // lsl r1, r2, #16
    fn lsl_r1_r2_16() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE1A0_1802);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(2, 0x0000_AA55);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(1), 0xAA55_0000);
    }

    #[test]
    // lsr r1, r2, #16
    fn lsr_r1_r2_16() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE1A0_1822);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(2, 0x00AA_AA55);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(1), 0x0000_00AA);
    }

    #[test]
    // asr r1, r2, #16
    fn asr_r1_r2_16() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE1A0_1842);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(2, 0x80AA_AA55);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(1), 0xFFFF_80AA);
    }

    #[test]
    // rrx r2, r1
    fn rrx_r2_r1() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE1A0_2061);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(1, 0x00AA_AA55);
        arm.cpsr.set_C(true);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(2), 0x8055_552A);
    }

    #[test]
    // ror r1, r2, #16
    fn ror_r1_r2_16() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE1A0_1862);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(2, 0x00AA_AA55);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(1), 0xAA55_00AA);
    }

    #[test]
    // bic r1, r2, r3
    fn bic_r1_r2_r3() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE1C2_1003);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(2, 0x00AA_AA55);
        arm.set_gpr(3, 0x00AA_AAAA);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(1), 0x0000_0055);
    }

    #[test]
    // mvn r1, r2
    fn mvn_r1_r2() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE1E0_1002);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(2, 0x00AA_AA55);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(1), 0xFF55_55AA);
    }

    #[test]
    // mul r1, r2, r3
    fn mul_r1_r2_r3() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE001_0392);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(2, 0xF000_0000);
        arm.set_gpr(3, 2);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(1), 0xE000_0000);
    }

    #[test]
    // mla r1, r2, r3, r4
    fn mla_r1_r2_r3_r4() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE021_4392);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(2, 0xF000_0000);
        arm.set_gpr(3, 2);
        arm.set_gpr(4, 0xAA55);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(1), 0xE000_AA55);
    }

    #[test]
    // umull r1, r2, r3, r4
    fn umull_r1_r2_r3_r4() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE082_1493);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(3, 0x7000_0001);
        arm.set_gpr(4, 0x0070_0000);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(1), 0x0070_0000);
        assert_eq!(arm.get_gpr(2), 0x0031_0000);
    }

    #[test]
    // umlal r1, r2, r3, r4
    fn umlal_r1_r2_r3_r4() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE0A2_1493);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(1, 0x0000_0001);
        arm.set_gpr(2, 0x0000_0002);
        arm.set_gpr(3, 0x7000_0001);
        arm.set_gpr(4, 0x0070_0000);
        arm.run_immediately();
        // assert_eq!(arm.get_gpr(1), 0x0070_0001);
        assert_eq!(arm.get_gpr(2), 0x0031_0002);
    }

    #[test]
    // smull r1, r2, r3, r4
    fn smull_r1_r2_r3_r4() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE0C2_1493);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(3, 0xFFFF_FFFE);
        arm.set_gpr(4, 0x7FFF_FFFF);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(1), 0x0000_0002);
        assert_eq!(arm.get_gpr(2), 0xFFFF_FFFF);
    }

    #[test]
    // smlal r1, r2, r3, r4
    fn smlal_r1_r2_r3_r4() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE0E2_1493);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(1, 0xFFFF_FFFF);
        arm.set_gpr(2, 0xFFFF_FFFF);
        arm.set_gpr(3, 0xFFFF_FFFE);
        arm.set_gpr(4, 0x7FFF_FFFF);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(1), 0x0000_0001);
        assert_eq!(arm.get_gpr(2), 0xFFFF_FFFF);
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
    // str r4, [r3]
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
    // strb r4, [r3]
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
    // strh r1, [r2]
    fn strh_r1_r2() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE1C2_10B0);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(2, 0x200);
        arm.set_gpr(1, 0x1155_55AA);
        arm.run_immediately();
        assert_eq!(arm.get_mem(0x200), 0x0000_55AA);
    }

    #[test]
    // strh r1, [r2, 0xff]
    fn strh_r1_r2_0xff() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE1C2_1FBF);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(2, 0x200);
        arm.set_gpr(1, 0x1155_55AA);
        arm.run_immediately();
        assert_eq!(arm.get_mem(0x2FF), 0x0000_55AA);
    }

    #[test]
    // ldrh r1, [r2]
    fn ldrh_r1_r2() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE1D2_10B0);
        &bus.set(0x200, 0xA5A5_5A5A);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(2, 0x200);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(1), 0x0000_5A5A);
    }

    #[test]
    // ldrsb r1, [r2]
    fn ldrsb_r1_r2() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE1D2_10D0);
        &bus.set(0x200, 0xA5A5_5AFF);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(2, 0x200);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(1), 0xFFFF_FFFF);
    }

    #[test]
    // ldrsh r1, [r2]
    fn ldrsh_r1_r2() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0, 0xE1D2_10D0);
        &bus.set(0x200, 0xA5A5_FFFE);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(2, 0x200);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(1), 0xFFFF_FFFE);
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

    #[test]
    // ldm r0!, {r4-r11}
    // Load 8 words from the source
    fn ldm_r0_r4_r11() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0000_0000, 0xE8B0_0FF0);
        for i in 0..0x10 {
            &bus.set(0x100 + (i * 4), 0xA000_0000 + i);
        }
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(0, 0x100);
        arm.run_immediately();
        assert_eq!(arm.get_gpr(PC), 0x0000_000C);
        assert_eq!(arm.get_gpr(0), 0x0000_0120);
        assert_eq!(arm.get_gpr(4), 0xA000_0000);
        assert_eq!(arm.get_gpr(5), 0xA000_0001);
        assert_eq!(arm.get_gpr(6), 0xA000_0002);
        assert_eq!(arm.get_gpr(7), 0xA000_0003);
        assert_eq!(arm.get_gpr(8), 0xA000_0004);
        assert_eq!(arm.get_gpr(9), 0xA000_0005);
        assert_eq!(arm.get_gpr(10), 0xA000_0006);
        assert_eq!(arm.get_gpr(11), 0xA000_0007);
        assert_eq!(arm.get_gpr(12), 0x0000_0000);
    }

    #[test]
    // stm r0!, {r4-r11}
    // Store 8 words from the source
    fn stm_r0_r4_r11() {
        setup();
        let mut bus = MockBus::new();
        &bus.set(0x0000_0000, 0xE8A0_0FF0);
        let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
        arm.set_gpr(0, 0x100);
        for i in 0..8 {
            arm.set_gpr(4 + i, 0xA000_0000 + i as u32);
        }
        arm.run_immediately();
        assert_eq!(arm.get_gpr(PC), 0x0000_000C);
        assert_eq!(arm.get_gpr(0), 0x0000_0120);
        assert_eq!(arm.get_mem(0x0000_0100), 0xA000_0000);
        assert_eq!(arm.get_mem(0x0000_0104), 0xA000_0001);
        assert_eq!(arm.get_mem(0x0000_0108), 0xA000_0002);
        assert_eq!(arm.get_mem(0x0000_010C), 0xA000_0003);
        assert_eq!(arm.get_mem(0x0000_0110), 0xA000_0004);
        assert_eq!(arm.get_mem(0x0000_0114), 0xA000_0005);
        assert_eq!(arm.get_mem(0x0000_0118), 0xA000_0006);
        assert_eq!(arm.get_mem(0x0000_011c), 0xA000_0007);
    }
}
