use anyhow::Context;

use crate::{
    instruction::Instruction,
    memory::Memory,
    registers::{Cond, R8, Registers},
    utils::{
        RegisterU16Ext, SetBit, half_carry_add_u8, half_carry_add_u16, half_carry_sub_u8,
        is_bit_set_u8,
    },
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

            instr @ (IncR8 { reg } | DecR8 { reg }) => {
                let val = self.get_r8(reg)?;

                let is_add = matches!(instr, Instruction::IncR8 { .. });
                let (new_val, carry_flag) = inc_or_dec(val, is_add);
                self.set_r8(reg, new_val);

                self.registers.set_z_flg(new_val == 0);
                self.registers.set_n_flg(!is_add);
                self.registers.set_h_flg(carry_flag);
            }

            LdR8Imm8 { reg } => {
                let value = self.imm8()?;
                self.set_r8(reg, value);
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

            Daa => {
                let a = self.registers.a();
                let first_digit = a & 0b1111;
                let subtract = self.registers.n_flg();

                let mut offset = 0;
                let mut carry = false;
                if (!subtract && first_digit > 9) || self.registers.h_flg() {
                    offset += 6;
                }

                if (!subtract && a > 0x99) | self.registers.c_flg() {
                    offset += 0x60;
                    carry = true;
                }

                let val = if subtract {
                    a.wrapping_sub(offset)
                } else {
                    a.wrapping_add(offset)
                };
                self.registers.set_a(val);

                self.registers.set_z_flg(val == 0);
                self.registers.set_h_flg(false);
                self.registers.set_c_flg(carry);
            }

            Cpl => {
                let a = self.registers.a();
                self.registers.set_a(!a);

                self.registers.set_n_flg(true);
                self.registers.set_h_flg(true);
            }

            Scf => {
                self.registers.set_n_flg(false);
                self.registers.set_h_flg(false);
                self.registers.set_c_flg(true);
            }

            Ccf => {
                self.registers.set_n_flg(false);
                self.registers.set_h_flg(false);
                self.registers.set_c_flg(!self.registers.c_flg());
            }

            JrImm8 => {
                let imm8_signed = self.imm8_signed()?;
                self.registers.pc = self
                    .registers
                    .pc
                    .wrapping_add_signed(i16::from(imm8_signed));
            }

            JrCondImm8 { cond } => {
                let imm8_signed = self.imm8_signed()?;
                if self.cond_met(cond) {
                    self.registers.pc = self
                        .registers
                        .pc
                        .wrapping_add_signed(i16::from(imm8_signed));

                    return Ok(Status::Cycles(3));
                }
            }

            LdR8R8 { src, dst } => {
                // TODO: Maybe move into `Instruction` or two match arms
                if src != dst {
                    self.set_r8(dst, self.get_r8(src)?);
                }
            }

            AddAR8 { reg, carry } => {
                let reg_val = self.get_r8(reg)?;
                let a = self.registers.a();

                let (mut new_val, mut overflow) = reg_val.overflowing_add(a);
                if carry && self.registers.c_flg() {
                    let (v, o) = new_val.overflowing_add(1);
                    new_val = v;
                    overflow |= o;
                }

                self.registers.set_a(new_val);

                self.registers.set_z_flg(new_val == 0);
                self.registers.set_n_flg(false);
                self.registers.set_h_flg(half_carry_add_u8(
                    reg_val,
                    a,
                    carry && self.registers.c_flg(),
                ));
                self.registers.set_c_flg(overflow);
            }

            SubAR8 { reg, carry } => {
                let reg_val = self.get_r8(reg)?;
                let a = self.registers.a();

                let (mut new_val, mut overflow) = a.overflowing_sub(reg_val);
                if carry && self.registers.c_flg() {
                    let (v, o) = new_val.overflowing_sub(1);
                    new_val = v;
                    overflow |= o;
                }

                self.registers.set_a(new_val);

                self.registers.set_z_flg(new_val == 0);
                self.registers.set_n_flg(true);
                self.registers.set_h_flg(half_carry_sub_u8(
                    a,
                    reg_val,
                    carry && self.registers.c_flg(),
                ));
                self.registers.set_c_flg(overflow);
            }

            AndAR8 { reg } => {
                let reg_val = self.get_r8(reg)?;
                let a = self.registers.a();
                let new_val = reg_val & a;

                self.registers.set_a(new_val);

                self.registers.set_z_flg(new_val == 0);
                self.registers.set_n_flg(false);
                self.registers.set_h_flg(true);
                self.registers.set_c_flg(false);
            }

            XorAR8 { reg } => {
                let reg_val = self.get_r8(reg)?;
                let a = self.registers.a();
                let new_val = reg_val ^ a;

                self.registers.set_a(new_val);

                self.registers.set_z_flg(new_val == 0);
                self.registers.set_n_flg(false);
                self.registers.set_h_flg(false);
                self.registers.set_c_flg(false);
            }

            OrAR8 { reg } => {
                let reg_val = self.get_r8(reg)?;
                let a = self.registers.a();
                let new_val = reg_val | a;

                self.registers.set_a(new_val);

                self.registers.set_z_flg(new_val == 0);
                self.registers.set_n_flg(false);
                self.registers.set_h_flg(false);
                self.registers.set_c_flg(false);
            }

            CpAR8 { reg } => {
                let reg_val = self.get_r8(reg)?;
                let a = self.registers.a();

                let (_, overflow) = a.overflowing_sub(reg_val);

                self.registers.set_z_flg(reg_val == a);
                self.registers.set_n_flg(true);
                self.registers
                    .set_h_flg(half_carry_sub_u8(a, reg_val, false));
                self.registers.set_c_flg(overflow);
            }

            RetCond { cond } => {
                if self.cond_met(cond) {
                    self.ret()?;
                    return Ok(Status::Cycles(5));
                }

            }

            Ret => {
                self.ret()?;
            }

            Reti => {
                // TODO: There's some interrupt stuff to do here
                self.ret()?;
            }

            JpCondImm16 { cond } => {
                let imm16 = self.imm16()?;

                if self.cond_met(cond) {
                    self.registers.pc = imm16;
                }

            }

            _ => anyhow::bail!(
                "Haven't implented instruction: {:08b} (0x{:x})",
                instruction_byte,
                instruction_byte
            ),
        };

        Ok(Status::Cycles(instruction.cycles()))
    }

    fn cond_met(&mut self, cond: Cond) -> bool {
        match cond {
            Cond::NZ => !self.registers.z_flg(),
            Cond::Z => self.registers.z_flg(),
            Cond::NC => !self.registers.c_flg(),
            Cond::C => self.registers.c_flg(),
        }
    }

    fn ret(&mut self) -> Result<(), anyhow::Error> {
        let low = self.memory.get_byte(self.registers.sp)?;
        let high = self.memory.get_byte(self.registers.sp.wrapping_add(1))?;
        self.registers.sp = self.registers.sp.wrapping_add(2);
        self.registers.pc = u16::from_be_bytes([high, low]);
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

    fn imm8(&mut self) -> anyhow::Result<u8> {
        self.next().context("Unable to read next byte for imm8")
    }

    fn imm8_signed(&mut self) -> anyhow::Result<i8> {
        Ok(self.imm8()? as i8)
    }

    // TODO: Test these
    pub fn get_r8(&self, r8: R8) -> anyhow::Result<u8> {
        Ok(match r8 {
            R8::B => self.registers.b(),
            R8::C => self.registers.c(),
            R8::D => self.registers.d(),
            R8::E => self.registers.e(),
            R8::H => self.registers.h(),
            R8::L => self.registers.l(),
            R8::A => self.registers.a(),
            R8::HL => self.memory.get_byte(self.registers.hl)?,
        })
    }

    pub fn set_r8(&mut self, r8: R8, value: u8) {
        match r8 {
            R8::B => self.registers.bc.set_high(value),
            R8::C => self.registers.bc.set_low(value),
            R8::D => self.registers.de.set_high(value),
            R8::E => self.registers.de.set_low(value),
            R8::H => self.registers.hl.set_high(value),
            R8::L => self.registers.hl.set_low(value),
            R8::A => self.registers.af.set_high(value),
            R8::HL => self.memory.set_byte(self.registers.hl, value),
        };
    }
}

// TODO: This is maybe a little dumb but it cleans up the code above.
fn inc_or_dec(value: u8, add: bool) -> (u8, bool) {
    if add {
        (value.wrapping_add(1), half_carry_add_u8(value, 1, false))
    } else {
        (value.wrapping_sub(1), half_carry_sub_u8(value, 1, false))
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
