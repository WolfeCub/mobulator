use mobulator_macros::opcode_match;

use crate::{
    byte_instruction::ByteInstruction,
    instructions::*,
    registers::{Cond, R8, R16, R16Mem},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Instruction {
    Nop,
    Halt,
    LdR16Imm16 { reg: R16 },
    LdR16memA { reg: R16Mem },
    LdAR16mem { reg: R16Mem },
    LdImm16Sp,
    IncR16 { reg: R16 },
    DecR16 { reg: R16 },
    AddHlR16 { reg: R16 },
    IncR8 { reg: R8 },
    DecR8 { reg: R8 },
    LdR8Imm8 { reg: R8 },
    Rlca,
    Rrca,
    Rla,
    Rra,
    Daa,
    Cpl,
    Scf,
    Ccf,
    JrImm8,
    JrCondImm8 { cond: Cond },
    LdR8R8 { src: R8, dst: R8 },
    AddAR8 { reg: R8, carry: bool },
}

impl TryFrom<u8> for Instruction {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let instruction = ByteInstruction(value);
        Ok(match value {
            NOOP => Instruction::Nop,
            HALT => Instruction::Halt,

            // ld r16, imm16
            opcode_match!(00__0001) => Instruction::LdR16Imm16 {
                reg: instruction.p().try_into()?,
            },

            // ld [r16mem], a
            opcode_match!(00__0010) => Instruction::LdR16memA {
                reg: instruction.p().try_into()?,
            },

            // ld a, [r16mem]
            opcode_match!(00__1010) => Instruction::LdAR16mem {
                reg: instruction.p().try_into()?,
            },

            // ld [imm16], sp
            LD_IMM16_SP => Instruction::LdImm16Sp,

            // inc r16
            opcode_match!(00__0011) => Instruction::IncR16 {
                reg: instruction.p().try_into()?,
            },

            // dec r16
            opcode_match!(00__1011) => Instruction::DecR16 {
                reg: instruction.p().try_into()?,
            },

            // add hl, r16
            opcode_match!(00__1001) => Instruction::AddHlR16 {
                reg: instruction.p().try_into()?,
            },

            // inc r8
            opcode_match!(00___100) => Instruction::IncR8 {
                reg: instruction.y().try_into()?,
            },
            // dec r8
            opcode_match!(00___101) => Instruction::DecR8 {
                reg: instruction.y().try_into()?,
            },

            // ld r8, imm8
            opcode_match!(00___110) => Instruction::LdR8Imm8 {
                reg: instruction.y().try_into()?,
            },

            RLCA => Instruction::Rlca,

            RRCA => Instruction::Rrca,

            RLA => Instruction::Rla,

            RRA => Instruction::Rra,

            DAA => Instruction::Daa,

            CPL => Instruction::Cpl,

            SCF => Instruction::Scf,

            CCF => Instruction::Ccf,

            JR_IMM8 => Instruction::JrImm8,

            // jr cond, imm8
            opcode_match!(001__000) => Instruction::JrCondImm8 {
                cond: instruction.cond().try_into()?,
            },

            // ld r8, r8
            opcode_match!(01______) => Instruction::LdR8R8 {
                src: instruction.z().try_into()?,
                dst: instruction.y().try_into()?,
            },

            // add a, r8
            // adc a, r8
            opcode_match!(1000____) => Instruction::AddAR8 {
                reg: instruction.z().try_into()?,
                carry: instruction.q(),
            },

            _ => anyhow::bail!(
                "Haven't implented instruction: {:08b} (0x{:x})",
                value,
                value
            ),
        })
    }
}

impl Instruction {
    pub fn cycles(&self) -> u8 {
        match self {
            Instruction::Nop => 1,
            Instruction::Halt => 1,
            Instruction::LdR16Imm16 { .. } => 3,
            Instruction::LdR16memA { .. } => 2,
            Instruction::LdAR16mem { .. } => 2,
            Instruction::LdImm16Sp => 5,
            Instruction::IncR16 { .. } => 2,
            Instruction::DecR16 { .. } => 2,
            Instruction::AddHlR16 { .. } => 2,
            Instruction::IncR8 { reg: R8::HL } => 3,
            Instruction::DecR8 { reg: R8::HL } => 3,
            Instruction::IncR8 { .. } => 1,
            Instruction::DecR8 { .. } => 1,
            Instruction::LdR8Imm8 { reg: R8::HL } => 3,
            Instruction::LdR8Imm8 { .. } => 2,
            Instruction::Rlca => 1,
            Instruction::Rrca => 1,
            Instruction::Rla => 1,
            Instruction::Rra => 1,
            Instruction::Daa => 1,
            Instruction::Cpl => 1,
            Instruction::Scf => 1,
            Instruction::Ccf => 1,
            Instruction::JrImm8 => 3,
            Instruction::JrCondImm8 { .. } => 2,
            Instruction::LdR8R8 { src: R8::HL, .. } => 2,
            Instruction::LdR8R8 { dst: R8::HL, .. } => 2,
            Instruction::LdR8R8 { .. } => 1,
            Instruction::AddAR8 { reg: R8::HL, .. } => 2,
            Instruction::AddAR8 { .. } => 1,
        }
    }
}
