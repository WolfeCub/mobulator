use mobulator_macros::opcode_match;

use crate::{
    byte_instruction::ByteInstruction,
    instructions::*,
    registers::{Cond, R16Mem, R16Stk, R16, R8},
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
    SubAR8 { reg: R8, carry: bool },
    AndAR8 { reg: R8 },
    XorAR8 { reg: R8 },
    OrAR8 { reg: R8 },
    CpAR8 { reg: R8 },
    RetCond { cond: Cond },
    Ret,
    Reti,
    JpCondImm16 { cond: Cond },
    JpImm16,
    JpHl,
    CallCondImm16 { cond: Cond },
    CallImm16,
    RstTgt3 { tgt3: u8 },
    PopR16stk { reg: R16Stk },
    PushR16stk { reg: R16Stk },
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

            // sub a, r8
            // sbc a, r8
            opcode_match!(1001____) => Instruction::SubAR8 {
                reg: instruction.z().try_into()?,
                carry: instruction.q(),
            },

            // and a, r8
            opcode_match!(10100___) => Instruction::AndAR8 {
                reg: instruction.z().try_into()?,
            },

            // xor a, r8
            opcode_match!(10101___) => Instruction::XorAR8 {
                reg: instruction.z().try_into()?,
            },

            // or a, r8
            opcode_match!(10110___) => Instruction::OrAR8 {
                reg: instruction.z().try_into()?,
            },

            // cp a, r8
            opcode_match!(10111___) => Instruction::CpAR8 {
                reg: instruction.z().try_into()?,
            },

            // ret cond
            opcode_match!(110__000) => Instruction::RetCond { cond: instruction.cond().try_into()? },

            // ret
            RET => Instruction::Ret,

            // reti
            RETI => Instruction::Reti,

            // jp cond, imm16
            opcode_match!(110__010) => Instruction::JpCondImm16 { cond: instruction.cond().try_into()? },

            // jp imm16
            JP_IMM16 => Instruction::JpImm16,

            // jp hl
            JP_HL => Instruction::JpHl,

            // call cond, imm16
            opcode_match!(110__100) => Instruction::CallCondImm16 { cond: instruction.cond().try_into()? },

            // call imm16
            CALL_IMM16 => Instruction::CallImm16,

            // rst tgt3
            opcode_match!(11___111) => Instruction::RstTgt3 { tgt3: instruction.y() },

            // pop r16stk
            opcode_match!(11__0001) => Instruction::PopR16stk { reg: instruction.p().try_into()? },

            // push r16stk
            opcode_match!(11__0101) => Instruction::PushR16stk { reg: instruction.p().try_into()? },

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
            Instruction::SubAR8 { reg: R8::HL, .. } => 2,
            Instruction::SubAR8 { .. } => 1,
            Instruction::AndAR8 { reg: R8::HL } => 2,
            Instruction::AndAR8 { .. } => 1,
            Instruction::XorAR8 { reg: R8::HL } => 2,
            Instruction::XorAR8 { .. } => 1,
            Instruction::OrAR8 { reg: R8::HL } => 2,
            Instruction::OrAR8 { .. } => 1,
            Instruction::CpAR8 { reg: R8::HL } => 2,
            Instruction::CpAR8 { .. } => 1,
            Instruction::RetCond { .. } => 2,
            Instruction::Ret => 4,
            Instruction::Reti => 4,
            Instruction::JpCondImm16 { .. } => 3,
            Instruction::JpImm16 => 4,
            Instruction::JpHl => 1,
            Instruction::CallCondImm16 { .. } => 3,
            Instruction::CallImm16 => 6,
            Instruction::RstTgt3 { .. } => 4,
            Instruction::PopR16stk { .. } => 3,
            Instruction::PushR16stk { .. } => 4,
        }
    }
}
