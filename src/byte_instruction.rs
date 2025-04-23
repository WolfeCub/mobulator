use crate::utils::is_bit_set_u8;

/// ┌───┬───┬───┬───┬───┬───┬───┬───┐
/// │ 7 │ 6 │ 5 │ 4 │ 3 │ 2 │ 1 │ 0 │
/// └───┴───┴───┴───┴───┴───┴───┴───┘
///   └───┘   └───────┘   └───────┘
///     x         y           z
///           └───┘   |
///             p     q
#[derive(Debug, Clone, Copy)]
pub struct ByteInstruction(pub u8);

// http://z80.info/decoding.htm
// x = the opcode's 1st octal digit (i.e. bits 7-6)
// y = the opcode's 2nd octal digit (i.e. bits 5-3)
// z = the opcode's 3rd octal digit (i.e. bits 2-0)
// p = y rightshifted one position (i.e. bits 5-4)
// q = y modulo 2 (i.e. bit 3)
impl ByteInstruction {
    pub fn x(&self) -> u8 {
        self.0 >> 6
    }

    pub fn y(&self) -> u8 {
        (self.0 & 0b00111111) >> 3
    }

    pub fn z(&self) -> u8 {
        self.0 & 0b00000111
    }

    pub fn p(&self) -> u8 {
        (self.0 >> 4) & 0b00000011
    }

    pub fn q(&self) -> bool {
        is_bit_set_u8(self.0, 3)
    }

    pub fn cond(&self) -> u8 {
        (self.0 >> 3) & 0b00000011
    }
}

#[cfg(test)]
mod tests {
    use crate::byte_instruction::ByteInstruction;

    #[test]
    fn decoder() {
        // 01_110_010
        //  x|  y|  z
        //    110
        //    p|q
        let segs = ByteInstruction(0b01_110_010);
        assert_eq!(segs.x(), 0b01);
        assert_eq!(segs.y(), 0b110);
        assert_eq!(segs.z(), 0b010);
        assert_eq!(segs.p(), 0b11);
        assert_eq!(segs.q(), false);

        // 11_101_101
        //  x|  y|  z
        //    101
        //    p|q
        let segs = ByteInstruction(0b11_101_101);
        assert_eq!(segs.x(), 0b11);
        assert_eq!(segs.y(), 0b101);
        assert_eq!(segs.z(), 0b101);
        assert_eq!(segs.p(), 0b10);
        assert_eq!(segs.q(), true);
    }
}
