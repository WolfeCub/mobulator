macro_rules! high {
    ($expression:expr) => {
        ($expression >> 8) as u8
    };
}

macro_rules! is_bit_set {
    ($expression:expr, $number:expr) => {
        ($expression & calc_nth_bit_power($number)) != 0
    };
}

const fn calc_nth_bit_power(bit: u32) -> u16 {
    2u16.pow(bit - 1)
}

#[derive(Debug, Clone, Default)]
pub struct Registers {
    // F register contains flags
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,
    // Stack pointer
    pub sp: u16,
    // Program counter
    pub pc: u16,
}

impl Registers {
    pub fn a(&self) -> u8 {
        high!(self.af)
    }

    pub fn z_flg(&self) -> bool {
        dbg!(calc_nth_bit_power(7));
        is_bit_set!(self.af, 7)
    }

    pub fn n_flg(&self) -> bool {
        is_bit_set!(self.af, 6)
    }

    pub fn h_flg(&self) -> bool {
        is_bit_set!(self.af, 5)
    }

    pub fn c_flg(&self) -> bool {
        is_bit_set!(self.af, 4)
    }

    pub fn b(&self) -> u8 {
        high!(self.bc)
    }

    pub fn c(&self) -> u8 {
        self.bc as u8
    }

    pub fn d(&self) -> u8 {
        high!(self.de)
    }

    pub fn e(&self) -> u8 {
        self.de as u8
    }

    pub fn h(&self) -> u8 {
        high!(self.hl)
    }

    pub fn l(&self) -> u8 {
        self.hl as u8
    }
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
            bc: 1290,
            ..Default::default()
        };

        assert_eq!(r.b(), 5);
        assert_eq!(r.c(), 10);
    }

    #[test]
    fn flags() {
        // 00000000_01000000 = 64
        let r = Registers {
            af: 64,
            ..Default::default()
        };

        assert_eq!(r.z_flg(), true);
        assert_eq!(r.n_flg(), false);
        assert_eq!(r.h_flg(), false);
        assert_eq!(r.c_flg(), false);

        let r = Registers {
            af: 91, // 01011011
            ..Default::default()
        };

        assert_eq!(r.z_flg(), true);
        assert_eq!(r.n_flg(), false);
        assert_eq!(r.h_flg(), true);
        assert_eq!(r.c_flg(), true);
    }
}
