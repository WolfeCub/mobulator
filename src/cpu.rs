use anyhow::Context;

use crate::{
    registers::Registers,
    utils::is_bit_set_u8,
};

#[derive(Debug, Clone)]
pub struct Cpu {
    registers: Registers,
    memory: Memory,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            registers: Registers::default(),
            memory: Memory::new(),
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

                // halt
                InstructionSegments {
                    x: 1, y: 6, z: 6, ..
                } => {
                    return Ok(());
                }

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

                    let joint = u16::from_le_bytes([first_byte, second_byte]);
                    self.registers.set_r16(p.try_into()?, joint);
                }

                // ld [r16mem], a
                InstructionSegments {
                    x: 0,
                    q: false,
                    z: 2,
                    p,
                    ..
                } => {
                    let addr = self.registers.get_r16(p.try_into()?);

                    let Some(value) = self.memory.get_byte(addr) else {
                        anyhow::bail!("Attempted to access out of bounds memory address: {:x}", addr);
                    };

                    // self.registers.a
                }

                _ => todo!(
                    "Haven't implented instruction: {:08b}, segments: {:?}",
                    instruction,
                    segments
                ),
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
        self.memory.get_byte(index)
    }
}

/// 0x0000 - 0x00FF: Boot ROM
/// 0x0000 - 0x3FFF: Game ROM Bank 0
/// 0x4000 - 0x7FFF: Game ROM Bank N
/// 0x8000 - 0x97FF: Tile RAM
/// 0x9800 - 0x9FFF: Background Map
/// 0xA000 - 0xBFFF: Cartridge RAM
/// 0xC000 - 0xDFFF: Working RAM
/// 0xE000 - 0xFDFF: Echo RAM
/// 0xFE00 - 0xFE9F: OAM (Object Atribute Memory)
/// 0xFEA0 - 0xFEFF: Unused
/// 0xFF00 - 0xFF7F: I/O Registers
/// 0xFF80 - 0xFFFE: High RAM Area
/// 0xFFFF: Interrupt Enabled Register
#[derive(Debug, Clone)]
struct Memory {
    memory: [u8; 0xFFFF],
}

impl Memory {
    pub fn new() -> Self {
        Self {
            memory: [0; 0xFFFF],
        }
    }

    pub fn get_byte(&self, index: u16) -> Option<u8> {
        self.memory.get(usize::from(index)).copied()
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
            cpu.memory.memory[..4].copy_from_slice(&[
                instruction,
                // Swapped order. Little endian
                0b00111100,
                0b10111100,
                // Halt
                0b01110110,
            ]);
            cpu.process_instructions()
                .expect("Unable to process CPU instructions");

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
