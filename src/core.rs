use std::cell::RefCell;
use std::error;
use std::fmt;
use std::rc::Rc;
use std::result::Result::Err;

use decoder::arm;
use instructions::shift;
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
    fn write_byte(&mut self, addr: u32, data: u8);
    fn write_word(&mut self, addr: u32, data: u32);
}

fn exec_memory_processing<F>(
    gpr: &mut [u32; 16],
    dec: &arm::Decoder,
    f: F,
) -> Result<PipelineStatus, ArmError>
where
    F: Fn(&mut [u32; 16], u32),
{
    let mut base = gpr[dec.get_Rn()];
    // INFO: Treat as imm12 if not I.
    let offset = if !dec.has_I() {
        dec.get_src2() as u32
    } else {
        let rm = dec.get_Rm() as usize;
        let sh = dec.get_sh();
        let shamt5 = dec.get_shamt5();
        match sh {
            arm::Shift::LSL => shift::lsl(gpr[rm], shamt5),
            arm::Shift::LSR => shift::lsr(gpr[rm], shamt5),
            arm::Shift::ASR => shift::asr(gpr[rm], shamt5),
            arm::Shift::ROR => shift::ror(gpr[rm], shamt5),
        }
    };
    let offset_base = if dec.is_plus_offset() {
        (base + offset) as Word
    } else {
        (base - offset) as Word
    };
    if dec.is_pre_indexed() {
        base = offset_base;
    }
    println!("base = {} rd = {}", base, dec.get_Rd());
    f(gpr, base);
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

/*
fn ldr<T: Bus>(
    gpr: &mut [u32; 16],
    bus: &Rc<RefCell<T>>,
    dec: arm::Decoder,
) -> Result<PipelineStatus, ArmError> {
    memory_process(gpr, dec, &|gpr, base, Rd| {
        gpr[Rd] = bus.borrow().read_word(base);
    })
}
*/

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
        let pipeline_status = match dec.opcode {
            arm::Opcode::LDR => self.exec_ldr(&dec)?,
            arm::Opcode::STR => self.exec_str(&dec)?,
            arm::Opcode::LDRB => self.exec_ldrb(&dec)?,
            arm::Opcode::STRB => self.exec_strb(&dec)?,
            arm::Opcode::MOV => self.exec_mov(dec)?,
            arm::Opcode::B => self.exec_b(dec)?,
            arm::Opcode::BL => self.exec_bl(dec)?,
            //arm::Opcode::Undefined => unimplemented!(),
            //arm::Opcode::NOP => unimplemented!(),
            //// arm::Opcode::SWI => unimplemented!(),
            // ArmOpcode::Unknown => self.execute_unknown(dec),
            _ => unimplemented!(),
        };

        match pipeline_status {
            PipelineStatus::Continue => self.increment_pc(),
            PipelineStatus::Flush => self.flush_pipeline(),
        };
        Ok(())
    }

    #[allow(non_snake_case)]
    fn exec_ldrb(&mut self, dec: &arm::Decoder) -> Result<PipelineStatus, ArmError> {
        let bus = &self.bus;
        exec_memory_processing(&mut self.gpr, &dec, |gpr, base| {
            let Rd = dec.get_Rd();
            gpr[Rd] = bus.borrow().read_byte(base) as Word;
        })
    }

    #[allow(non_snake_case)]
    fn exec_ldr(&mut self, dec: &arm::Decoder) -> Result<PipelineStatus, ArmError> {
        let bus = &self.bus;
        exec_memory_processing(&mut self.gpr, &dec, |gpr, base| {
            gpr[dec.get_Rd()] = bus.borrow().read_word(base);
        })
    }

    fn exec_str(&mut self, dec: &arm::Decoder) -> Result<PipelineStatus, ArmError> {
        let bus = &self.bus;
        exec_memory_processing(&mut self.gpr, &dec, |gpr, base| {
            bus.borrow_mut().write_word(base, gpr[dec.get_Rd()]);
        })
    }

    fn exec_strb(&mut self, dec: &arm::Decoder) -> Result<PipelineStatus, ArmError> {
        let bus = &self.bus;
        exec_memory_processing(&mut self.gpr, &dec, |gpr, base| {
            bus.borrow_mut().write_byte(base, gpr[dec.get_Rd()] as Byte);
        })
    }
    fn exec_mov(&mut self, dec: arm::Decoder) -> Result<PipelineStatus, ArmError> {
        if dec.has_I() {
            let src2 = dec.get_src2();
            self.gpr[dec.get_Rd()] = src2 as Word;
        } else {
            // TODO: implement later
        }
        Ok(PipelineStatus::Continue)
    }

    fn exec_bl(&mut self, dec: arm::Decoder) -> Result<PipelineStatus, ArmError> {
        self.gpr[LR] = self.gpr[PC] - 4;
        self.exec_b(dec)
    }

    fn exec_b(&mut self, dec: arm::Decoder) -> Result<PipelineStatus, ArmError> {
        let imm = dec.get_imm24() as u32;
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
