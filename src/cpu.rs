use anyhow::Context;

use crate::{
    instruction::Instruction,
    memory::Memory,
    registers::Registers,
    utils::{half_carry_add_u16, half_carry_add_u8, half_carry_sub_u8, is_bit_set_u8, SetBit},
};

#[derive(Debug, Clone, Default)]
pub struct Cpu {
    pub registers: Registers,
    pub memory: Memory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Cycles(u8),
    Break,
}

impl Cpu {
    pub fn run_num_instructions(&mut self, num: u8) -> anyhow::Result<()> {
        for _ in 0..num {
            self.run_next_instruction()?;
        }
        Ok(())
    }

    pub fn run_next_instruction(&mut self) -> anyhow::Result<Status> {
        let Some(instruction_byte) = self.next() else {
            return Ok(Status::Break);
        };

        let instruction = Instruction::try_from(instruction_byte)?;

        use Instruction::*;

        match instruction {
            Nop => (),
            LdAA => (),

            LdR16Imm16 { reg } => {
                let data = self.imm16()?;
                self.registers.set_r16(reg, data);
            }

            LdR16memA { reg } => {
                let addr = self.registers.get_r16mem(reg);
                self.memory.set_byte(addr, self.registers.a());
            }

            LdAR16mem { reg } => {
                let addr = self.registers.get_r16mem(reg);

                let data = self.memory.get_byte(addr)?;
                self.registers.set_a(data);
            }

            LdImm16Sp => {
                let addr = self.imm16()?;
                let [high, low] = self.registers.sp.to_be_bytes();
                self.memory.set_byte(addr, low);
                self.memory.set_byte(addr + 1, high);
            }

            IncR16 { reg } => {
                let reg_val = self.registers.get_r16(reg);
                let new_val = reg_val.wrapping_add(1);
                self.registers.set_r16(reg, new_val);
            }

            DecR16 { reg } => {
                let reg_val = self.registers.get_r16(reg);
                let new_val = reg_val.wrapping_sub(1);
                self.registers.set_r16(reg, new_val);
            }

            AddHlR16 { reg } => {
                let reg_val = self.registers.get_r16(reg);
                let (result, overflow) = self.registers.hl.overflowing_add(reg_val);

                self.registers.set_n_flg(false);
                self.registers
                    .set_h_flg(half_carry_add_u16(reg_val, self.registers.hl));
                self.registers.set_c_flg(overflow);

                self.registers.hl = result;
            }

            instr@(IncHl | DecHl) => {
                let addr = self.registers.hl;
                let byte = self.memory.get_byte(addr)?;

                let (value, carry_flag) = inc_or_dec(byte, instr == IncHl);
                self.memory.set_byte(addr, value);

                self.registers.set_z_flg(value == 0);
                self.registers.set_n_flg(instr == DecHl);
                self.registers.set_h_flg(carry_flag);
            }
            instr@(IncR8 { reg } | DecR8 { reg }) => {
                let val = self.registers.get_r8(reg);

                let is_add = matches!(instr, Instruction::IncR8 {..});
                let (new_val, carry_flag) = inc_or_dec(val, is_add);
                self.registers.set_r8(reg, new_val);

                self.registers.set_z_flg(new_val == 0);
                self.registers.set_n_flg(!is_add);
                self.registers.set_h_flg(carry_flag);
            }

            LdHlImm8 => {
                let value = self.next().context("Unable to read byte after imm8")?;
                self.memory.set_byte(self.registers.hl, value);
            }
            LdR8Imm8 { reg } => {
                let value = self.next().context("Unable to read byte after imm8")?;
                self.registers.set_r8(reg, value);
            }

            Rlca => {
                let rotated = self.registers.a().rotate_left(1);
                self.registers.set_a(rotated);

                self.registers.set_z_flg(false);
                self.registers.set_n_flg(false);
                self.registers.set_h_flg(false);
                self.registers.set_c_flg(is_bit_set_u8(rotated, 0));
            }

            Rrca => {
                let a = self.registers.a();
                let rotated = a.rotate_right(1);
                self.registers.set_a(rotated);

                self.registers.set_z_flg(false);
                self.registers.set_n_flg(false);
                self.registers.set_h_flg(false);
                self.registers.set_c_flg(is_bit_set_u8(a, 0));
            }

            Rla => {
                let a = self.registers.a();
                let mut rotated = a.rotate_left(1);

                rotated.set_bit(0, self.registers.c_flg());
                self.registers.set_a(rotated);

                self.registers.set_z_flg(false);
                self.registers.set_n_flg(false);
                self.registers.set_h_flg(false);
                self.registers.set_c_flg(is_bit_set_u8(a, 7));
            }

            Rra => {
                let a = self.registers.a();
                let mut rotated = a.rotate_right(1);

                rotated.set_bit(7, self.registers.c_flg());
                self.registers.set_a(rotated);

                self.registers.set_z_flg(false);
                self.registers.set_n_flg(false);
                self.registers.set_h_flg(false);
                self.registers.set_c_flg(is_bit_set_u8(a, 0));
            }

            // Daa => {}

            _ => anyhow::bail!(
                "Haven't implented instruction: {:08b} (0x{:x})",
                instruction_byte,
                instruction_byte
            ),
        };

        Ok(Status::Cycles(instruction.cycles()))
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

// TODO: This is maybe a little dumb but it cleans up the code above.
fn inc_or_dec(value: u8, add: bool) -> (u8, bool) {
    if add {
        (value.wrapping_add(1), half_carry_add_u8(value, 1))
    } else {
        (value.wrapping_sub(1), half_carry_sub_u8(value, 1))
    }
}

impl Iterator for Cpu {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let byte = self.memory.get_byte(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);
        byte.ok()
    }
}

#[cfg(test)]
mod tests {
    use std::u16;

    use crate::{
        byte_instruction::ByteInstruction, instructions::*, registers::{R16, R8}
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
            ]);
            cpu.run_next_instruction()
                .expect("Unable to process CPU instructions");

            let p = ByteInstruction(instruction).p();
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
            cpu.memory.load_instructions(&[instruction]);
            cpu.registers.set_a(0b10110101);
            let addr = 0xDC17; // 0xC000 - 0xDFFF working mem

            // HL+ and HL- both access HL
            let p = ByteInstruction(instruction).p();
            let p = if p == 3 { 2 } else { p };

            cpu.registers.set_r16(p.try_into().unwrap(), addr);
            cpu.run_next_instruction()
                .expect("Unable to process CPU instructions");

            assert_eq!(cpu.memory.get_byte(addr).expect("Byte exists"), 0b10110101);
        }
    }

    #[test]
    fn ld_a_r16mem() {
        // ld a, [r16mem]
        for instruction in opcode_list!(00__1010) {
            let mut cpu = Cpu::default();
            cpu.memory.load_instructions(&[instruction]);

            let addr = 0xDC17; // 0xC000 - 0xDFFF working mem

            // HL+ and HL- both access HL
            let p = ByteInstruction(instruction).p();
            let p = if p == 3 { 2 } else { p };

            cpu.registers.set_r16(p.try_into().unwrap(), addr);
            cpu.memory.set_byte(addr, 0x6F);

            cpu.run_next_instruction()
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
            .load_instructions(&[LD_IMM16_SP, first, second]);

        cpu.registers.sp = 0b11101011_10001001;

        cpu.run_next_instruction()
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
            cpu.memory.load_instructions(&[instruction]);

            let instruction = ByteInstruction(instruction);
            let reg = instruction.p().try_into().expect("Invalid r16");
            cpu.registers.set_r16(reg, 1337);

            cpu.run_next_instruction()
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
            cpu.memory.load_instructions(&[instruction]);

            let instruction = ByteInstruction(instruction);
            let reg = instruction.p().try_into().expect("Invalid r16");
            cpu.registers.set_r16(reg, 1337);
            cpu.registers.hl = 2424;

            cpu.run_next_instruction()
                .expect("Unable to process CPU instructions");

            // if r16 is register HL then we double our value rather than adding from another reg
            let target = match reg {
                R16::HL => 2424 + 2424,
                _ => 1337 + 2424,
            };
            assert_eq!(cpu.registers.hl, target);
            assert_eq!(cpu.registers.n_flg(), false);
            assert_eq!(cpu.registers.c_flg(), false);
            assert_eq!(cpu.registers.c_flg(), false);
        }
    }

    #[test]
    fn add_hl_r16_flags() {
        let mut cpu = Cpu::default();
        cpu.memory.load_instructions(&[0b00001001]);

        cpu.registers.bc = u16::MAX;
        cpu.registers.hl = 2;

        cpu.run_next_instruction()
            .expect("Unable to process CPU instructions");

        assert_eq!(cpu.registers.hl, 1);
        assert_eq!(cpu.registers.c_flg(), true);
        assert_eq!(cpu.registers.h_flg(), true);

        let mut cpu = Cpu::default();
        cpu.memory.load_instructions(&[0b00001001]);
        cpu.registers.bc = 62 << 8;
        cpu.registers.hl = 34 << 8;

        cpu.run_next_instruction()
            .expect("Unable to process CPU instructions");

        assert_eq!(cpu.registers.h_flg(), true);

        let mut cpu = Cpu::default();
        cpu.memory.load_instructions(&[0b00001001]);
        cpu.registers.bc = 1;
        cpu.registers.hl = 2;

        cpu.run_next_instruction()
            .expect("Unable to process CPU instructions");

        assert_eq!(cpu.registers.h_flg(), false);
    }

    #[test]
    fn inc_dec_r8() {
        // inc r8
        // dec r8
        for instruction in [opcode_list!(00___100), opcode_list!(00___101)].concat() {
            let mut cpu = Cpu::default();
            cpu.memory.load_instructions(&[instruction]);
            cpu.registers.hl = 0xDC17;

            let instruction = ByteInstruction(instruction);
            // TODO: This should be nicer. Maybe split into two tests.
            let val = if instruction.y() != 6 {
                let reg = instruction.y().try_into().expect("Invalid r8");
                cpu.registers.set_r8(reg, 137);

                cpu.run_next_instruction()
                    .expect("Unable to process CPU instructions");

                cpu.registers.get_r8(reg)
            } else {
                cpu.memory.set_byte(cpu.registers.hl, 137);

                cpu.run_next_instruction()
                    .expect("Unable to process CPU instructions");

                cpu.memory
                    .get_byte(cpu.registers.hl)
                    .expect("Unable to get byte")
            };

            let (num, flg) = if instruction.0 % 2 == 1 {
                (136, true)
            } else {
                (138, false)
            };
            assert_eq!(val, num);
            assert_eq!(cpu.registers.n_flg(), flg);
        }
    }

    #[test]
    fn inc_dec_r8_flags() {
        // inc
        let mut cpu = Cpu::default();
        cpu.memory.load_instructions(&[0b00000100]);

        cpu.registers.set_r8(R8::B, 0b0000_1111);

        cpu.run_next_instruction()
            .expect("Unable to process CPU instructions");

        assert_eq!(cpu.registers.h_flg(), true);
        assert_eq!(cpu.registers.n_flg(), false);

        // dec
        let mut cpu = Cpu::default();
        cpu.memory.load_instructions(&[0b00000101]);

        cpu.registers.set_r8(R8::B, 0b0001_0000);

        cpu.run_next_instruction()
            .expect("Unable to process CPU instructions");

        assert_eq!(cpu.registers.h_flg(), true);
        assert_eq!(cpu.registers.n_flg(), true);
    }

    #[test]
    fn ld_r8_imm8() {
        // ld r8, imm8
        for instruction in opcode_list!(00___110) {
            let mut cpu = Cpu::default();
            cpu.memory
                .load_instructions(&[instruction, 0b00111100]);
            cpu.registers.hl = 0xDC17;

            let instruction = ByteInstruction(instruction);

            cpu.run_next_instruction()
                .expect("Unable to process CPU instructions");

            let val = if instruction.0 != LD_HL_IMM8 {
                cpu.registers.get_r8(instruction.y().try_into().unwrap())
            } else {
                cpu.memory.get_byte(cpu.registers.hl).unwrap()
            };

            assert_eq!(val, 0b00111100);
        }
    }

    #[test]
    fn rlca() {
        // rlca
        let mut cpu = Cpu::default();
        cpu.memory.load_instructions(&[RLCA]);

        cpu.registers.af = 0b10011000_11100000;

        cpu.run_next_instruction()
            .expect("Unable to process CPU instructions");

        assert_eq!(cpu.registers.a(), 0b00110001);
        assert_eq!(cpu.registers.z_flg(), false);
        assert_eq!(cpu.registers.n_flg(), false);
        assert_eq!(cpu.registers.h_flg(), false);
        assert_eq!(cpu.registers.c_flg(), true);

        let mut cpu = Cpu::default();
        cpu.memory.load_instructions(&[RLCA]);

        cpu.registers.af = 0b00011000_11100000;

        cpu.run_next_instruction()
            .expect("Unable to process CPU instructions");

        assert_eq!(cpu.registers.a(), 0b00110000);
        assert_eq!(cpu.registers.c_flg(), false);
    }

    #[test]
    fn rrca() {
        // rrca
        let mut cpu = Cpu::default();
        cpu.memory.load_instructions(&[RRCA]);

        cpu.registers.af = 0b10011000_11100000;

        cpu.run_next_instruction()
            .expect("Unable to process CPU instructions");

        assert_eq!(cpu.registers.a(), 0b01001100);
        assert_eq!(cpu.registers.z_flg(), false);
        assert_eq!(cpu.registers.n_flg(), false);
        assert_eq!(cpu.registers.h_flg(), false);
        assert_eq!(cpu.registers.c_flg(), false);

        let mut cpu = Cpu::default();
        cpu.memory.load_instructions(&[RRCA]);

        cpu.registers.af = 0b00011001_11100000;

        cpu.run_next_instruction()
            .expect("Unable to process CPU instructions");

        assert_eq!(cpu.registers.a(), 0b10001100);
        assert_eq!(cpu.registers.c_flg(), true);
    }

    #[test]
    fn rla() {
        // rla
        let mut cpu = Cpu::default();
        cpu.memory.load_instructions(&[RLA]);

        cpu.registers.af = 0b10011000_11100000;

        cpu.run_next_instruction()
            .expect("Unable to process CPU instructions");

        assert_eq!(cpu.registers.a(), 0b00110000);
        assert_eq!(cpu.registers.z_flg(), false);
        assert_eq!(cpu.registers.n_flg(), false);
        assert_eq!(cpu.registers.h_flg(), false);
        assert_eq!(cpu.registers.c_flg(), true);

        let mut cpu = Cpu::default();
        cpu.memory.load_instructions(&[RLA]);

        // Set carry flag here
        cpu.registers.af = 0b00011000_11110000;

        cpu.run_next_instruction()
            .expect("Unable to process CPU instructions");

        assert_eq!(cpu.registers.a(), 0b00110001);
        assert_eq!(cpu.registers.z_flg(), false);
        assert_eq!(cpu.registers.n_flg(), false);
        assert_eq!(cpu.registers.h_flg(), false);
        assert_eq!(cpu.registers.c_flg(), false);
    }

    #[test]
    fn rra() {
        // rra
        let mut cpu = Cpu::default();
        cpu.memory.load_instructions(&[RRA]);

        cpu.registers.af = 0b10011001_11100000;

        cpu.run_next_instruction()
            .expect("Unable to process CPU instructions");

        assert_eq!(cpu.registers.a(), 0b01001100);
        assert_eq!(cpu.registers.z_flg(), false);
        assert_eq!(cpu.registers.n_flg(), false);
        assert_eq!(cpu.registers.h_flg(), false);
        assert_eq!(cpu.registers.c_flg(), true);

        let mut cpu = Cpu::default();
        cpu.memory.load_instructions(&[RRA]);

        // Set carry flag here
        cpu.registers.af = 0b00011000_11110000;

        cpu.run_next_instruction()
            .expect("Unable to process CPU instructions");

        assert_eq!(cpu.registers.a(), 0b10001100);
        assert_eq!(cpu.registers.z_flg(), false);
        assert_eq!(cpu.registers.n_flg(), false);
        assert_eq!(cpu.registers.h_flg(), false);
        assert_eq!(cpu.registers.c_flg(), false);
    }
}
