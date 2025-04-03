use anyhow::Context;

use crate::{
    registers::Registers,
    utils::{is_bit_set_u8, join_u8s},
};

#[derive(Debug, Clone)]
pub struct Cpu {
    registers: Registers,
    instructions: Vec<u8>,
    memory: [u8; 0xFFFF],
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            registers: Registers::default(),
            instructions: Vec::new(),
            memory: [0; 0xFFFF],
        }
    }

    pub fn process_instructions(&mut self) -> anyhow::Result<()> {
        while let Some(instruction) = self.next() {
            let segments = InstructionSegments::from_instruction(instruction);

            match segments {
                // NOOP
                InstructionSegments {
                    x: 0, y: 0, z: 0, ..
                } => (),

                // ld r16, imm16
                InstructionSegments {
                    x: 0,
                    q: false,
                    z: 1,
                    p,
                    ..
                } => {
                    let first_byte = self
                        .next()
                        .context("Unable to read next byte after imm16")?;
                    let second_byte = self
                        .next()
                        .context("Unable to read next byte after imm16")?;

                    let joint = join_u8s(second_byte, first_byte);
                    self.registers.set_r16(p.try_into()?, joint);
                },

                // ld [r16mem], a
                InstructionSegments {
                    x: 0,
                    q: false,
                    z: 2,
                    p,
                    ..
                } => {

                },

                _ => todo!("Haven't implented instruction: {:08b}, segments: {:?}", instruction, segments),
            };
        }

        Ok(())
    }
}

impl Iterator for Cpu {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let memory_offset = 0;
        let index = (self.registers.pc - memory_offset) / 8;
        self.registers.pc += 8;
        self.instructions.get(index as usize).copied()
    }
}

/// ┌───┬───┬───┬───┬───┬───┬───┬───┐
/// │ 7 │ 6 │ 5 │ 4 │ 3 │ 2 │ 1 │ 0 │
/// └───┴───┴───┴───┴───┴───┴───┴───┘
///   └───┘   └───────┘   └───────┘
///     x         y           z
///           └───┘   |
///             p     q
#[derive(Debug, Clone)]
pub struct InstructionSegments {
    x: u8,
    y: u8,
    z: u8,
    p: u8,
    q: bool,
}

impl InstructionSegments {
    pub fn from_instruction(instruction: u8) -> Self {
        // http://z80.info/decoding.htm
        // x = the opcode's 1st octal digit (i.e. bits 7-6)
        // y = the opcode's 2nd octal digit (i.e. bits 5-3)
        // z = the opcode's 3rd octal digit (i.e. bits 2-0)
        // p = y rightshifted one position (i.e. bits 5-4)
        // q = y modulo 2 (i.e. bit 3)
        let x = instruction >> 6;
        let y = (instruction & 63) >> 3; // 00111111 = 63 (discard top 2 bits)
        let z = instruction & 7; // 00000111 = 63 (discard top 5 bits)
        let p = y >> 1;
        let q = is_bit_set_u8(y, 0);

        Self { x, y, z, p, q }
    }
}

#[cfg(test)]
mod tests {
    use crate::{cpu::InstructionSegments, registers::R16};

    use super::Cpu;

    #[test]
    fn decoder() {
        // 01_110_010
        //  x|  y|  z
        //    110
        //    p|q
        let segs = InstructionSegments::from_instruction(0b01_110_010);
        assert_eq!(segs.x, 0b01);
        assert_eq!(segs.y, 0b110);
        assert_eq!(segs.z, 0b010);
        assert_eq!(segs.p, 0b11);
        assert_eq!(segs.q, false);

        // 11_101_101
        //  x|  y|  z
        //    101
        //    p|q
        let segs = InstructionSegments::from_instruction(0b11_101_101);
        assert_eq!(segs.x, 0b11);
        assert_eq!(segs.y, 0b101);
        assert_eq!(segs.z, 0b101);
        assert_eq!(segs.p, 0b10);
        assert_eq!(segs.q, true);
    }

    #[test]
    fn ld_r16_imm16() {
        // 00_[r16]0_001
        for i in 0..4 {
            let instruction = u8::from_str_radix(&format!("00{:02b}0001", i), 2)
                .expect("Unable to parse generated number");

            let mut cpu = Cpu::new();
            cpu.instructions = vec![
                instruction,
                // Swapped order. Little endian
                0b00111100,
                0b10111100,
            ];
            cpu.process_instructions().expect("Unable to process CPU instructions");

            let target = match R16::try_from(i).expect("Used invalid R16 register") {
                R16::BC => cpu.registers.bc,
                R16::DE => cpu.registers.de,
                R16::HL => cpu.registers.hl,
                R16::SP => cpu.registers.sp,
            };
            assert_eq!(target, 0b10111100_00111100);
        }
    }
}
