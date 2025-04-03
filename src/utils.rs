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

pub(crate) const fn high_u8(number: u16) -> u8 {
    (number >> 8) as u8
}

pub(crate) const fn join_u8s(one: u8, two: u8) -> u16 {
    (one as u16) << 8 | two as u16
}


// TODO: Expand these
#[cfg(test)]
mod tests {
    use crate::utils::join_u8s;

    #[test]
    fn test_join_u8s() {
        // 00100011_11001110
        assert_eq!(join_u8s(0b00100011, 0b11001110), 0b00100011_11001110);

        // 01100011_11100100
        assert_eq!(join_u8s(0b01100011, 0b11100100), 0b01100011_11100100);
    }
}
