// https://matklad.github.io/2021/07/09/inline-in-rust.html
// "it usually isn’t necessary to apply #[inline] to private functions — within a crate, the
// compiler generally makes good inline decisions."

pub(crate) const fn calc_nth_bit_power(bit: u32) -> u16 {
    2u16.pow(bit)
}

pub(crate) const fn is_bit_set_u8(number: u8, bit: u32) -> bool {
    is_bit_set_u16(number as u16, bit)
}

pub(crate) const fn is_bit_set_u16(number: u16, bit: u32) -> bool {
    (number & calc_nth_bit_power(bit)) != 0
}

pub(crate) const fn half_carry_add_u8(a: u8, b: u8) -> bool {
    (((a & 0xF) + (b & 0xF)) & 0x10) == 0x10
}

pub(crate) const fn half_carry_add_u16(a: u16, b: u16) -> bool {
    (((a & 0xFFF) + (b & 0xFFF)) & 0x1000) == 0x1000
}

pub(crate) const fn half_carry_sub_u8(a: u8, b: u8) -> bool {
    a & 0x0F < b & 0x0F
}

pub(crate) const fn half_carry_sub_u16(a: u16, b: u16) -> bool {
    a & 0x0F < b & 0x0F
}

pub trait RegisterU16Ext {
    fn set_high(&mut self, value: u8);
    fn set_low(&mut self, value: u8);
    fn high_u8(&self) -> u8;
    fn set_bit(&mut self, bit: u32, value: bool);
}

impl RegisterU16Ext for u16 {
    fn set_high(&mut self, value: u8) {
        *self &= 0b00000000_11111111;
        *self |= (value as u16) << 8;
    }

    fn set_low(&mut self, value: u8) {
        *self &= 0b11111111_00000000;
        *self |= value as u16;
    }

    fn high_u8(&self) -> u8 {
        let [high, _] = self.to_be_bytes();
        high
    }

    fn set_bit(&mut self, bit: u32, value: bool) {
        if value {
            *self |= calc_nth_bit_power(bit);
        } else {
            *self &= !calc_nth_bit_power(bit);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::utils::RegisterU16Ext;

    #[test]
    fn test_set_high() {
        let mut target: u16 = 0b10110001_10001110;
        let new_val = 0b11101010;

        target.set_high(new_val);
        let [high, low] = target.to_be_bytes();

        assert_eq!(high, new_val);
        assert_eq!(low, 0b10001110);
    }

    #[test]
    fn test_set_low() {
        let mut target: u16 = 0b10110001_10001110;
        let new_val = 0b11101010;

        target.set_low(new_val);
        let [high, low] = target.to_be_bytes();

        assert_eq!(low, new_val);
        assert_eq!(high, 0b10110001);
    }
}
