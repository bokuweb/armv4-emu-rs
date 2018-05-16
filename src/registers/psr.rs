use std::mem;

/// The CPU's instruction decoding states.
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum State {
    /// Currently executing 32-bit ARM instructions.
    ARM = 0,

    /// Currently executing 16-bit THUMB instructions.
    THUMB,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Mode {
    User = 0,
    FIQ,
    IRQ,
    Supervisor,
    Abort,
    Undefined,
    System,
}

impl Mode {
    pub fn as_bits(self) -> u32 {
        match self {
            Mode::User => PSR::MODE_USER,
            Mode::FIQ => PSR::MODE_FIQ,
            Mode::IRQ => PSR::MODE_IRQ,
            Mode::Supervisor => PSR::MODE_SUPERVISOR,
            Mode::Abort => PSR::MODE_ABORT,
            Mode::Undefined => PSR::MODE_UNDEFINED,
            Mode::System => PSR::MODE_SYSTEM,
        }
    }
}

/// The Program Status Register.
#[derive(PartialEq, Clone, Copy)]
pub struct PSR(pub u32);

impl PSR {
    const RAW_DEFAULT: u32 =
        PSR::MODE_SUPERVISOR | (1 << PSR::IRQ_DISABLE_BIT) | (1 << PSR::FIQ_DISABLE_BIT);

    const NON_RESERVED_MASK: u32 = 0b11110000_00000000_00000000_11111111_u32;
    //                               NZCV                       IFTMMMMM

    const FLAGS_MASK: u32 = 0xF0000000_u32;
    const N_FLAG_BIT: u32 = 31;
    const Z_FLAG_BIT: u32 = 30;
    const C_FLAG_BIT: u32 = 29;
    const V_FLAG_BIT: u32 = 28;

    const IRQ_DISABLE_BIT: u32 = 7;
    const FIQ_DISABLE_BIT: u32 = 6;

    const STATE_BIT: u32 = 5;

    const MODE_MASK: u32 = 0b0001_1111;
    const MODE_USER: u32 = 0b1_0000;
    const MODE_FIQ: u32 = 0b1_0001;
    const MODE_IRQ: u32 = 0b1_0010;
    const MODE_SUPERVISOR: u32 = 0b1_0011;
    const MODE_ABORT: u32 = 0b1_0111;
    const MODE_UNDEFINED: u32 = 0b1_1011;
    const MODE_SYSTEM: u32 = 0b1_1111;

    pub fn clear_reserved_bits(&mut self) {
        self.0 &= PSR::NON_RESERVED_MASK;
    }

    pub fn state(&self) -> State {
        unsafe { mem::transmute(((self.0 >> PSR::STATE_BIT) & 1) as u8) }
    }

    pub fn mode(&self) -> Mode {
        match self.0 & PSR::MODE_MASK {
            PSR::MODE_USER => Mode::User,
            PSR::MODE_FIQ => Mode::FIQ,
            PSR::MODE_IRQ => Mode::IRQ,
            PSR::MODE_SUPERVISOR => Mode::Supervisor,
            PSR::MODE_ABORT => Mode::Abort,
            PSR::MODE_UNDEFINED => Mode::Undefined,
            PSR::MODE_SYSTEM => Mode::System,
            _ => {
                error!(
                    "PSR: Unrecognised mode bit pattern {:#010b}.",
                    self.0 & PSR::MODE_MASK
                );
                panic!("Aborting due to illegal mode bits.");
            }
        }
    }

    pub fn set_state(&mut self, s: State) {
        self.0 &= !(1 << PSR::STATE_BIT);
        self.0 |= (s as u8 as u32) << PSR::STATE_BIT;
    }

    pub fn set_mode(&mut self, m: Mode) {
        self.0 &= !PSR::MODE_MASK;
        self.0 |= m.as_bits();
    }

    pub fn disable_irq(&mut self) {
        self.0 |= 1 << PSR::IRQ_DISABLE_BIT;
    }

    pub fn disable_fiq(&mut self) {
        self.0 |= 1 << PSR::FIQ_DISABLE_BIT;
    }

    pub fn enable_irq(&mut self) {
        self.0 &= !(1 << PSR::IRQ_DISABLE_BIT);
    }

    pub fn enable_fiq(&mut self) {
        self.0 &= !(1 << PSR::FIQ_DISABLE_BIT);
    }

    pub fn irq_disabled(&self) -> bool {
        (self.0 & (1 << PSR::IRQ_DISABLE_BIT)) != 0
    }

    pub fn fiq_disabled(&self) -> bool {
        (self.0 & (1 << PSR::FIQ_DISABLE_BIT)) != 0
    }

    #[allow(non_snake_case)]
    pub fn get_N(self) -> bool {
        (self.0 & (1 << PSR::N_FLAG_BIT)) != 0
    }

    #[allow(non_snake_case)]
    pub fn get_Z(self) -> bool {
        (self.0 & (1 << PSR::Z_FLAG_BIT)) != 0
    }

    #[allow(non_snake_case)]
    pub fn get_C(self) -> bool {
        (self.0 & (1 << PSR::C_FLAG_BIT)) != 0
    }

    #[allow(non_snake_case)]
    pub fn get_V(self) -> bool {
        (self.0 & (1 << PSR::V_FLAG_BIT)) != 0
    }

    #[allow(non_snake_case)]
    pub fn set_N(&mut self, n: bool) {
        self.0 = (self.0 & !(1 << PSR::N_FLAG_BIT)) | ((n as u32) << PSR::N_FLAG_BIT);
    }

    #[allow(non_snake_case)]
    pub fn set_Z(&mut self, n: bool) {
        self.0 = (self.0 & !(1 << PSR::Z_FLAG_BIT)) | ((n as u32) << PSR::Z_FLAG_BIT);
    }

    #[allow(non_snake_case)]
    pub fn set_C(&mut self, n: bool) {
        self.0 = (self.0 & !(1 << PSR::C_FLAG_BIT)) | ((n as u32) << PSR::C_FLAG_BIT);
    }

    #[allow(non_snake_case)]
    pub fn set_V(&mut self, n: bool) {
        self.0 = (self.0 & !(1 << PSR::V_FLAG_BIT)) | ((n as u32) << PSR::V_FLAG_BIT);
    }

    pub fn set(&mut self, d: u32) {
        self.0 = d;
    }
}

impl Default for PSR {
    fn default() -> PSR {
        PSR(PSR::RAW_DEFAULT)
    }
}
