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
    let [high, _] = number.to_be_bytes();
    high
}
