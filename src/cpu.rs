use anyhow::Context;
use mobulator_macros::opcode_match;

use crate::{instructions::*, registers::Registers, utils::is_bit_set_u8};

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
        while let Some(instruction_byte) = self.next() {
            let instruction = Instruction(instruction_byte);
            match instruction_byte {
                NOOP => (),

                HALT => {
                    return Ok(());
                }

                // ld r16, imm16
                opcode_match!(00__0001) => {
                    let data = self.imm16()?;
                    self.registers.set_r16(instruction.p().try_into()?, data);
                }

                // ld [r16mem], a
                opcode_match!(00__0010) => {
                    let addr = self.registers.get_r16(instruction.p().try_into()?);
                    self.memory.set_byte(addr, self.registers.a());
                }

                // ld a, [r16mem]
                opcode_match!(00__1010) => {
                    let addr = self.registers.get_r16(instruction.p().try_into()?);

                    let Some(data) = self.memory.get_byte(addr) else {
                        anyhow::bail!("Out of bounds memory access at {:x}", addr);
                    };
                    self.registers.set_a(data);
                }

                // ld [imm16], sp
                LD_IMM16_SP => {
                    let addr = self.imm16()?;
                    let [high, low] = self.registers.sp.to_be_bytes();
                    self.memory.set_byte(addr, low);
                    self.memory.set_byte(addr + 8, high);
                }

                _ => todo!("Haven't implented instruction: {:08b}", instruction_byte,),
            };
        }

        Ok(())
    }

    fn imm16(&mut self) -> anyhow::Result<u16> {
        let first_byte = self
            .next()
            .context("Unable to read first byte after imm16")?;
        let second_byte = self
            .next()
            .context("Unable to read second byte after imm16")?;
        let joint = u16::from_le_bytes([first_byte, second_byte]);
        Ok(joint)
    }
}

impl Iterator for Cpu {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let byte = self.memory.get_byte(self.registers.pc);
        self.registers.pc += 8;
        byte
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

    pub fn get_byte(&self, addr: u16) -> Option<u8> {
        self.memory.get(usize::from(addr / 8)).copied()
    }

    pub fn set_byte(&mut self, addr: u16, value: u8) {
        self.memory[usize::from(addr / 8)] = value;
    }
}

/// ┌───┬───┬───┬───┬───┬───┬───┬───┐
/// │ 7 │ 6 │ 5 │ 4 │ 3 │ 2 │ 1 │ 0 │
/// └───┴───┴───┴───┴───┴───┴───┴───┘
///   └───┘   └───────┘   └───────┘
///     x         y           z
///           └───┘   |
///             p     q
#[derive(Debug, Clone, Copy)]
pub struct Instruction(u8);

// http://z80.info/decoding.htm
// x = the opcode's 1st octal digit (i.e. bits 7-6)
// y = the opcode's 2nd octal digit (i.e. bits 5-3)
// z = the opcode's 3rd octal digit (i.e. bits 2-0)
// p = y rightshifted one position (i.e. bits 5-4)
// q = y modulo 2 (i.e. bit 3)
impl Instruction {
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
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::Instruction,
        instructions::{HALT, LD_IMM16_SP},
        registers::R16,
    };
    use mobulator_macros::opcode_list;

    use super::Cpu;

    #[test]
    fn decoder() {
        // 01_110_010
        //  x|  y|  z
        //    110
        //    p|q
        let segs = Instruction(0b01_110_010);
        assert_eq!(segs.x(), 0b01);
        assert_eq!(segs.y(), 0b110);
        assert_eq!(segs.z(), 0b010);
        assert_eq!(segs.p(), 0b11);
        assert_eq!(segs.q(), false);

        // 11_101_101
        //  x|  y|  z
        //    101
        //    p|q
        let segs = Instruction(0b11_101_101);
        assert_eq!(segs.x(), 0b11);
        assert_eq!(segs.y(), 0b101);
        assert_eq!(segs.z(), 0b101);
        assert_eq!(segs.p(), 0b10);
        assert_eq!(segs.q(), true);
    }

    #[test]
    fn ld_r16_imm16() {
        // 00_[r16]0_001
        for instruction in opcode_list!(00__0001) {
            let mut cpu = Cpu::new();
            cpu.memory.memory[..4].copy_from_slice(&[
                instruction,
                // Swapped order. Little endian
                0b00111100,
                0b10111100,
                HALT,
            ]);
            cpu.process_instructions()
                .expect("Unable to process CPU instructions");

            let p = Instruction(instruction).p();
            let target = match R16::try_from(p).expect("Used invalid R16 register") {
                R16::BC => cpu.registers.bc,
                R16::DE => cpu.registers.de,
                R16::HL => cpu.registers.hl,
                R16::SP => cpu.registers.sp,
            };
            assert_eq!(target, 0b10111100_00111100);
        }
    }

    #[test]
    fn ld_r16mem_a() {
        // ld [r16mem], a
        for instruction in opcode_list!(00__0010) {
            let mut cpu = Cpu::new();
            cpu.memory.memory[..2].copy_from_slice(&[instruction, HALT]);
            cpu.registers.set_a(0b10110101);
            let addr = 0xDC17; // 0xC000 - 0xDFFF working mem

            let p = Instruction(instruction).p();
            cpu.registers.set_r16(p.try_into().unwrap(), addr);
            cpu.process_instructions()
                .expect("Unable to process CPU instructions");

            let mem_val = cpu.memory.memory[usize::from(addr / 8)];
            assert_eq!(mem_val, 0b10110101);
        }
    }

    #[test]
    fn ld_a_r16mem() {
        // ld a, [r16mem]
        for instruction in opcode_list!(00__1010) {
            let mut cpu = Cpu::new();
            cpu.memory.memory[..2].copy_from_slice(&[instruction, HALT]);

            let addr = 0xDC17; // 0xC000 - 0xDFFF working mem

            let p = Instruction(instruction).p();
            cpu.registers.set_r16(p.try_into().unwrap(), addr);
            cpu.memory.memory[usize::from(addr / 8)] = 0x6F;

            cpu.process_instructions()
                .expect("Unable to process CPU instructions");

            assert_eq!(cpu.registers.a(), 0x6F);
        }
    }
    #[test]
    fn ld_imm16_sp() {
        // ld [imm16], sp
        let mut cpu = Cpu::new();

        let addr: u16 = 0xDC17; // 0xC000 - 0xDFFF working mem
        let [first, second] = addr.to_le_bytes();
        cpu.memory.memory[..4].copy_from_slice(&[LD_IMM16_SP, first, second, HALT]);

        cpu.registers.sp = 0b11101011_10001001;

        cpu.process_instructions()
            .expect("Unable to process CPU instructions");

        let first_mem_val = cpu.memory.memory[usize::from(addr / 8)];
        let second_mem_val = cpu.memory.memory[usize::from(addr / 8) + 1];
        assert_eq!(first_mem_val, 0b10001001);
        assert_eq!(second_mem_val, 0b11101011);
    }
}
