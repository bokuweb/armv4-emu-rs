use std::cell::RefCell;
use std::rc::Rc;

use instructions::arm;

pub const INITIAL_PIPELINE_WAIT: u8 = 2;
pub const PC_OFFSET: usize = 2;

enum Arm {
	NOP,
	NOP_RAW,
}

enum PipelineState {
	WAIT,
	Ready,
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

#[derive(Debug, PartialEq, Clone, Copy)]
struct PSR;

impl PSR {
	pub fn default() -> Self {
		PSR
	}
}

pub const PC: usize = 15;

struct CpuBus;

pub struct ARMv4<T> {
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
	fn read_word(&self, addr: u32) -> Word;
}

impl<T> ARMv4<T>
where
	T: Bus,
{
	pub const SP: usize = 13;
	pub const LR: usize = 14;
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

	fn execute(&mut self, inst: arm::Instruction) -> Result<(), ()> {
		debug!("decoded instruction = {:?}", inst);
		match inst.opcode {
			arm::Opcode::LDR => self.ldr(inst),
			arm::Opcode::STR => self.str(inst),
			arm::Opcode::MOV => self.mov(inst),
			arm::Opcode::B => self.branch(inst),
			arm::Opcode::BL => unimplemented!(),
			arm::Opcode::Undefined => unimplemented!(),
			arm::Opcode::NOP => unimplemented!(),
			// arm::Opcode::SWI => unimplemented!(),
			// ArmOpcode::Unknown => self.execute_unknown(inst),
		}
	}

	fn ldr(&mut self, inst: arm::Instruction) -> Result<(), ()> {
		let mut base = self.gpr[inst.get_Rn()];
		// INFO: Treat as imm12 if not I.
		if !inst.has_I() {
			let src2 = inst.get_src2() as i32;
			let offset = (src2 * if inst.is_plus_offset() { 1 } else { -1 }) as i32;
			base = (base as i32 + offset) as Word;
		} else {
			// TODO: implement later
		}
		self.gpr[inst.get_Rd()] = self.bus.borrow().read_word(base);
		Ok(())
	}

	fn str(&mut self, inst: arm::Instruction) -> Result<(), ()> {
		Ok(())
	}

	fn mov(&mut self, inst: arm::Instruction) -> Result<(), ()> {
		if inst.has_I() {
			let src2 = inst.get_src2();
			self.gpr[inst.get_Rd()] = src2 as Word;
		} else {
			// TODO: implement later
		}
		Ok(())
	}

	fn branch(&mut self, inst: arm::Instruction) -> Result<(), ()> {
		let imm = inst.get_imm24() as u32;
		let imm = (if imm & 0x0080_0000 != 0 {
			imm | 0xFF00_0000
		} else {
			imm
		}) as i32;
		self.gpr[PC] = (self.gpr[PC] as i32 + imm * 4) as Word;
		self.flush_pipeline();
		Ok(())
	}

	pub fn tick(&mut self) -> Result<(), ()> {
		if self.pipeline_wait > 0 {
			self.pipeline_wait -= 1;
			self.increment_pc();
			return Ok(());
		}
		match self.state {
			CpuState::ARM => {
				let fetched = self.bus.borrow().read_word(self.gpr[PC - PC_OFFSET]);
				debug!("fetched code = {:x}", fetched);
				let decoded = arm::Instruction::decode(fetched);
				self.execute(decoded)
			}
			// TODO: Thumb mode
			_ => unimplemented!(),
		}
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
	// ldr pc, =0x8000_0000
	fn ldr_pc_eq0x8000_0000() {
		setup();
		let mut bus = MockBus::new();
		&bus.set(0x0, 0xE51F_F004);
		&bus.set(0x4, 0x8000_0000);
		let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
		arm.run_immediately();
		assert_eq!(arm.gpr[PC], 0x8000_0000);
	}

	#[test]
	// mov r0, #1
	fn mov_r0_imm1() {
		setup();
		let mut bus = MockBus::new();
		&bus.set(0x0, 0xE3A0_0001);
		let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
		arm.run_immediately();
		assert_eq!(arm.gpr[0x00], 0x0000_0001);
	}

	#[test]
	// b pc-2
	fn b_pc_sub_3() {
		setup();
		let mut bus = MockBus::new();
		&bus.set(0x0000_0000, 0xEAFF_FFFE);
		let mut arm = ARMv4::new(Rc::new(RefCell::new(bus)));
		arm.run_immediately();
		assert_eq!(arm.gpr[PC], 0x0000_0000);
	}
}
