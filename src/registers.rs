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

    pub fn z_flg(&self) -> bool {
        is_bit_set(self.af, 7)
    }

    pub fn n_flg(&self) -> bool {
        is_bit_set(self.af, 6)
    }

    pub fn h_flg(&self) -> bool {
        is_bit_set(self.af, 5)
    }

    pub fn c_flg(&self) -> bool {
        is_bit_set(self.af, 4)
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
}

const fn calc_nth_bit_power(bit: u32) -> u16 {
    2u16.pow(bit)
}

#[inline]
fn is_bit_set(number: u16, bit: u32) -> bool {
    (number & calc_nth_bit_power(bit)) != 0
}

#[inline]
fn high_u8(number: u16) -> u8 {
    (number >> 8) as u8
}


// TODO: Test more thouroughly
#[cfg(test)]
mod tests {
    use super::Registers;

    #[test]
    fn b_registers() {
        // 00000101_00001010 = 1290
        //        5|      10
        //
        let r = Registers {
            af: 1290,
            bc: 1290,
            de: 1290,
            hl: 1290,
            ..Default::default()
        };

        assert_eq!(r.a(), 5);

        assert_eq!(r.b(), 5);
        assert_eq!(r.c(), 10);

        assert_eq!(r.d(), 5);
        assert_eq!(r.e(), 10);

        assert_eq!(r.h(), 5);
        assert_eq!(r.l(), 10);
    }

    #[test]
    fn flags() {
        // 00000000_10000000 = 128
        let r = Registers {
            af: 128,
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
