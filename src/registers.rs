use crate::utils::{is_bit_set_u16, RegisterU16Ext};

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
        self.af.high_u8()
    }

    pub fn set_a(&mut self, val: u8) {
        self.af.set_high(val);
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
        self.bc.high_u8()
    }

    pub fn c(&self) -> u8 {
        self.bc as u8
    }

    pub fn d(&self) -> u8 {
        self.de.high_u8()
    }

    pub fn e(&self) -> u8 {
        self.de as u8
    }

    pub fn h(&self) -> u8 {
        self.hl.high_u8()
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

    pub fn get_r16mem(&mut self, r16: R16Mem) -> u16 {
        match r16 {
            R16Mem::BC => self.bc,
            R16Mem::DE => self.de,
            R16Mem::HLI => {
                let val = self.hl;
                self.hl += 1;
                val
            }
            R16Mem::HLD => {
                let val = self.hl;
                self.hl -= 1;
                val
            }
        }
    }

    pub fn get_r8(&self, r8: R8) -> u8 {
        match r8 {
            R8::B => self.b(),
            R8::C => self.c(),
            R8::D => self.d(),
            R8::E => self.e(),
            R8::H => self.h(),
            R8::L => self.l(),
            R8::A => self.a(),
        }
    }

    pub fn set_r8(&mut self, r8: R8, value: u8) {
        match r8 {
            R8::B => self.bc.set_high(value),
            R8::C => self.bc.set_low(value),
            R8::D => self.de.set_high(value),
            R8::E => self.de.set_low(value),
            R8::H => self.hl.set_high(value),
            R8::L => self.hl.set_low(value),
            R8::A => self.af.set_high(value),
        };
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

#[derive(Clone, Copy, Debug)]
pub enum R8 {
    B,
    C,
    D,
    E,
    H,
    L,
    A,
}

impl TryFrom<u8> for R8 {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(R8::B),
            1 => Ok(R8::C),
            2 => Ok(R8::D),
            3 => Ok(R8::E),
            4 => Ok(R8::H),
            5 => Ok(R8::L),
            7 => Ok(R8::A),
            _ => anyhow::bail!("Unable to convert u8: '{value}' to R8"),
        }
    }
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
            _ => anyhow::bail!("Unable to convert u8: '{value}' to R16"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum R16Mem {
    BC,
    DE,
    HLI,
    HLD,
}

impl TryFrom<u8> for R16Mem {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(R16Mem::BC),
            1 => Ok(R16Mem::DE),
            2 => Ok(R16Mem::HLI),
            3 => Ok(R16Mem::HLD),
            _ => anyhow::bail!("Unable to convert u8: '{value}' to R16Mem"),
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
pub enum Cond {
    NZ = 0,
    Z = 1,
    NC = 2,
    C = 3,
}

// TODO: Test more thoroughly
#[cfg(test)]
mod tests {
    use crate::registers::{R16Mem, R8};

    use super::Registers;

    #[test]
    fn b_registers() {
        let mut r = Registers {
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

        r.set_r8(R8::C, 0b10111100);
        assert_eq!(r.bc, 0b00000101_10111100);

        r.set_r8(R8::D, 0b10111100);
        assert_eq!(r.de, 0b10111100_00001010);
    }

    #[test]
    fn flags() {
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

    #[test]
    fn get_hli_hld() {
        let mut r = Registers {
            hl: 78,
            ..Default::default()
        };

        assert_eq!(r.get_r16mem(R16Mem::HLI), 78);
        assert_eq!(r.hl, 79);

        assert_eq!(r.get_r16mem(R16Mem::HLD), 79);
        assert_eq!(r.hl, 78);
    }
}
