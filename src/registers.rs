use crate::utils::{high_u8, is_bit_set_u16, set_high_u8};

// https://gbdev.io/pandocs/CPU_Registers_and_Flags.html
#[derive(Debug, Clone, Default)]
pub struct Registers {
    // F register contains flags
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
    // Stack pointer
    pub sp: u16,
    // Program counter
    pub pc: u16,
}

impl Registers {
    pub fn a(&self) -> u8 {
        high_u8(self.af)
    }

    pub fn set_a(&mut self, val: u8) {
        set_high_u8(&mut self.af, val);
    }

    pub fn z_flg(&self) -> bool {
        is_bit_set_u16(self.af, 7)
    }

    pub fn n_flg(&self) -> bool {
        is_bit_set_u16(self.af, 6)
    }

    pub fn h_flg(&self) -> bool {
        is_bit_set_u16(self.af, 5)
    }

    pub fn c_flg(&self) -> bool {
        is_bit_set_u16(self.af, 4)
    }

    pub fn b(&self) -> u8 {
        high_u8(self.bc)
    }

    pub fn c(&self) -> u8 {
        self.bc as u8
    }

    pub fn d(&self) -> u8 {
        high_u8(self.de)
    }

    pub fn e(&self) -> u8 {
        self.de as u8
    }

    pub fn h(&self) -> u8 {
        high_u8(self.hl)
    }

    pub fn l(&self) -> u8 {
        self.hl as u8
    }

    pub fn set_r16(&mut self, r16: R16, val: u16) {
        match r16 {
            R16::BC => self.bc = val,
            R16::DE => self.de = val,
            R16::HL => self.hl = val,
            R16::SP => self.sp = val,
        }
    }

    pub fn get_r16(&self, r16: R16) -> u16 {
        match r16 {
            R16::BC => self.bc,
            R16::DE => self.de,
            R16::HL => self.hl,
            R16::SP => self.sp,
        }
    }

    pub fn get_r16_mut(&mut self, r16: R16) -> &mut u16 {
        match r16 {
            R16::BC => &mut self.bc,
            R16::DE => &mut self.de,
            R16::HL => &mut self.hl,
            R16::SP => &mut self.sp,
        }
    }
}

// ┌───────┬────┬────┬────┬────┬────┬────┬──────┬────┐
// │       │  0 │  1 │  2 │  3 │  4 │  5 │  6   │  7 │
// ├───────┼────┼────┼────┼────┼────┼────┼──────┼────┤
// │ r8    │ b  │ c  │ d  │ e  │ h  │ l  │ [hl] │ a  │
// │ r16   │ bc │ de │ hl │ sp │    │    │      │    │
// │ r16stk│ bc │ de │ hl │ af │    │    │      │    │
// │ r16mem│ bc │ de │ hl+│ hl-│    │    │      │    │
// │ cond  │ nz │ z  │ nc │ c  │    │    │      │    │
// ├───────┴────┴────┴────┴────┴────┴────┴──────┴────┤
// │ b3    │ A 3-bit bit index                       │
// │ tgt3  │ rst's target address, divided by 8      │
// │ imm8  │ The following byte                      │
// │ imm16 │ The following two bytes (little-endian) │
// └─────────────────────────────────────────────────┘
// TODO: Remove repr
#[repr(u8)]
pub enum R8 {
    B = 0,
    C = 1,
    D = 2,
    E = 3,
    H = 4,
    L = 5,
    HL = 6, // TODO: What is this '[hl]'
    A = 7,
}

#[derive(Clone, Copy, Debug)]
pub enum R16 {
    BC,
    DE,
    HL,
    SP,
}

impl TryFrom<u8> for R16 {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(R16::BC),
            1 => Ok(R16::DE),
            2 => Ok(R16::HL),
            3 => Ok(R16::SP),
            _ => anyhow::bail!("Unable to convert u16: '{value}' to R16"),
        }
    }
}

// TODO: Remove repr
#[repr(u8)]
pub enum R16Stk {
    BC = 0,
    DE = 1,
    HL = 2,
    AF = 3,
}

// TODO: Remove repr
#[repr(u8)]
pub enum R16Mem {
    BC = 0,
    DE = 1,
    HLI = 2,
    HLD = 3,
}

// TODO: Remove repr
#[repr(u8)]
pub enum Cond {
    NZ = 0,
    Z = 1,
    NC = 2,
    C = 3,
}

// TODO: Test more thoroughly
#[cfg(test)]
mod tests {
    use super::Registers;

    #[test]
    fn b_registers() {
        // 00000101_00001010
        let r = Registers {
            af: 0b00000101_00001010,
            bc: 0b00000101_00001010,
            de: 0b00000101_00001010,
            hl: 0b00000101_00001010,
            ..Default::default()
        };

        assert_eq!(r.a(), 0b00000101);

        assert_eq!(r.b(), 0b00000101);
        assert_eq!(r.c(), 0b00001010);

        assert_eq!(r.d(), 0b00000101);
        assert_eq!(r.e(), 0b00001010);

        assert_eq!(r.h(), 0b00000101);
        assert_eq!(r.l(), 0b00001010);
    }

    #[test]
    fn flags() {
        // 00000000_10000000
        let r = Registers {
            af: 0b00000000_10000000,
            ..Default::default()
        };

        assert_eq!(r.z_flg(), true);
        assert_eq!(r.n_flg(), false);
        assert_eq!(r.h_flg(), false);
        assert_eq!(r.c_flg(), false);

        let r = Registers {
            af: 176, // 10110000
            ..Default::default()
        };

        assert_eq!(r.z_flg(), true);
        assert_eq!(r.n_flg(), false);
        assert_eq!(r.h_flg(), true);
        assert_eq!(r.c_flg(), true);
    }
}
