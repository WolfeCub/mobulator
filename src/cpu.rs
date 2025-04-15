use anyhow::Context;
use mobulator_macros::opcode_match;

use crate::{instruction::Instruction, instructions::*, memory::Memory, registers::Registers};

#[derive(Debug, Clone, Default)]
pub struct Cpu {
    registers: Registers,
    memory: Memory,
}

impl Cpu {
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
                    let addr = self.registers.get_r16mem(instruction.p().try_into()?);
                    self.memory.set_byte(addr, self.registers.a());
                }

                // ld a, [r16mem]
                opcode_match!(00__1010) => {
                    let addr = self.registers.get_r16mem(instruction.p().try_into()?);

                    let data = self.memory.get_byte(addr)?;
                    self.registers.set_a(data);
                }

                // ld [imm16], sp
                LD_IMM16_SP => {
                    let addr = self.imm16()?;
                    let [high, low] = self.registers.sp.to_be_bytes();
                    self.memory.set_byte(addr, low);
                    self.memory.set_byte(addr + 1, high);
                }

                // inc r16
                opcode_match!(00__0011) => {
                    let reg_val = self.registers.get_r16_mut(instruction.p().try_into()?);
                    *reg_val += 1
                }

                // dec r16
                opcode_match!(00__1011) => {
                    let reg_val = self.registers.get_r16_mut(instruction.p().try_into()?);
                    *reg_val -= 1
                }

                // add hl, r16
                opcode_match!(00__1001) => {
                    self.registers.hl += self.registers.get_r16(instruction.p().try_into()?);
                }

                INC_HL => {
                    let addr = self.registers.hl;
                    let byte = self.memory.get_byte_mut(addr)?;
                    *byte += 1;
                }
                // inc r8
                opcode_match!(00___100) => {
                    let r8 = instruction.y().try_into()?;
                    let val = self.registers.get_r8(r8);
                    self.registers.set_r8(r8, val + 1);
                }

                DEC_HL => {
                    let addr = self.registers.hl;
                    let byte = self.memory.get_byte_mut(addr)?;
                    *byte -= 1;
                }
                // dec r8
                opcode_match!(00___101) => {
                    let r8 = instruction.y().try_into()?;
                    let val = self.registers.get_r8(r8);
                    self.registers.set_r8(r8, val - 1);
                }

                // ld r8, imm8
                LD_HL_IMM8 => {
                    let value = self.next().context("Unable to read byte after imm8")?;
                    self.memory.set_byte(self.registers.hl, value);
                }
                opcode_match!(00___110) => {
                    let value = self.next().context("Unable to read byte after imm8")?;
                    self.registers.set_r8(instruction.y().try_into()?, value);
                }

                // rlca
                // RCLA => {
                // }

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
        self.registers.pc += 1;
        byte.ok()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        instruction::Instruction,
        instructions::{HALT, LD_HL_IMM8, LD_IMM16_SP},
        registers::R16,
    };
    use mobulator_macros::opcode_list;

    use super::Cpu;

    #[test]
    fn ld_r16_imm16() {
        // 00_[r16]0_001
        for instruction in opcode_list!(00__0001) {
            let mut cpu = Cpu::default();
            cpu.memory.load_instructions(&[
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
            let mut cpu = Cpu::default();
            cpu.memory.load_instructions(&[instruction, HALT]);
            cpu.registers.set_a(0b10110101);
            let addr = 0xDC17; // 0xC000 - 0xDFFF working mem

            // HL+ and HL- both access HL
            let p = Instruction(instruction).p();
            let p = if p == 3 { 2 } else { p };

            cpu.registers.set_r16(p.try_into().unwrap(), addr);
            cpu.process_instructions()
                .expect("Unable to process CPU instructions");

            assert_eq!(cpu.memory.get_byte(addr).expect("Byte exists"), 0b10110101);
        }
    }

    #[test]
    fn ld_a_r16mem() {
        // ld a, [r16mem]
        for instruction in opcode_list!(00__1010) {
            let mut cpu = Cpu::default();
            cpu.memory.load_instructions(&[instruction, HALT]);

            let addr = 0xDC17; // 0xC000 - 0xDFFF working mem

            // HL+ and HL- both access HL
            let p = Instruction(instruction).p();
            let p = if p == 3 { 2 } else { p };

            cpu.registers.set_r16(p.try_into().unwrap(), addr);
            cpu.memory.set_byte(addr, 0x6F);

            cpu.process_instructions()
                .expect("Unable to process CPU instructions");

            assert_eq!(cpu.registers.a(), 0x6F);
        }
    }

    #[test]
    fn ld_imm16_sp() {
        // ld [imm16], sp
        let mut cpu = Cpu::default();

        let addr: u16 = 0xDC17; // 0xC000 - 0xDFFF working mem
        let [first, second] = addr.to_le_bytes();
        cpu.memory
            .load_instructions(&[LD_IMM16_SP, first, second, HALT]);

        cpu.registers.sp = 0b11101011_10001001;

        cpu.process_instructions()
            .expect("Unable to process CPU instructions");

        let first_mem_val = cpu.memory.get_byte(addr).expect("Byte exists");
        let second_mem_val = cpu.memory.get_byte(addr + 1).expect("Byte exists");
        assert_eq!(first_mem_val, 0b10001001);
        assert_eq!(second_mem_val, 0b11101011);
    }

    #[test]
    fn inc_dec_r16() {
        // inc r16
        // dec r16
        for instruction in opcode_list!(00___011) {
            let mut cpu = Cpu::default();
            cpu.memory.load_instructions(&[instruction, HALT]);

            let instruction = Instruction(instruction);
            let reg = instruction.p().try_into().expect("Invalid r16");
            cpu.registers.set_r16(reg, 1337);

            cpu.process_instructions()
                .expect("Unable to process CPU instructions");

            let num = if instruction.q() { 1336 } else { 1338 };
            assert_eq!(cpu.registers.get_r16(reg), num);
        }
    }

    #[test]
    fn add_hl_r16() {
        // add hl, r16
        for instruction in opcode_list!(00__1001) {
            let mut cpu = Cpu::default();
            cpu.memory.load_instructions(&[instruction, HALT]);

            let instruction = Instruction(instruction);
            let reg = instruction.p().try_into().expect("Invalid r16");
            cpu.registers.set_r16(reg, 1337);
            cpu.registers.hl = 2424;

            cpu.process_instructions()
                .expect("Unable to process CPU instructions");

            // if r16 is register HL then we double our value rather than adding from another reg
            let target = match reg {
                R16::HL => 2424 + 2424,
                _ => 1337 + 2424,
            };
            assert_eq!(cpu.registers.hl, target);
        }
    }

    #[test]
    fn inc_dec_r8() {
        // inc r8
        // dec r8
        for instruction in [opcode_list!(00___100), opcode_list!(00___101)].concat() {
            let mut cpu = Cpu::default();
            cpu.memory.load_instructions(&[instruction, HALT]);
            cpu.registers.hl = 0xDC17;

            let instruction = Instruction(instruction);
            // TODO: This should be nicer. Maybe split into two tests.
            let val = if instruction.y() != 6 {
                let reg = instruction.y().try_into().expect("Invalid r8");
                cpu.registers.set_r8(reg, 137);

                cpu.process_instructions()
                    .expect("Unable to process CPU instructions");

                cpu.registers.get_r8(reg)
            } else {
                cpu.memory.set_byte(cpu.registers.hl, 137);

                cpu.process_instructions()
                    .expect("Unable to process CPU instructions");

                cpu.memory
                    .get_byte(cpu.registers.hl)
                    .expect("Unable to get byte")
            };

            let num = if instruction.0 % 2 == 1 { 136 } else { 138 };
            assert_eq!(val, num);
        }
    }

    #[test]
    fn ld_r8_imm8() {
        // ld r8, imm8
        for instruction in opcode_list!(00___110) {
            let mut cpu = Cpu::default();
            cpu.memory
                .load_instructions(&[instruction, 0b00111100, HALT]);
            cpu.registers.hl = 0xDC17;

            let instruction = Instruction(instruction);

            cpu.process_instructions()
                .expect("Unable to process CPU instructions");

            let val = if instruction.0 != LD_HL_IMM8 {
                cpu.registers.get_r8(instruction.y().try_into().unwrap())
            } else {
                cpu.memory.get_byte(cpu.registers.hl).unwrap()
            };

            assert_eq!(val, 0b00111100);
        }
    }
}
