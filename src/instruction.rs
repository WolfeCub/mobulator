use mobulator_macros::opcode_match;

use crate::{
    byte_instruction::ByteInstruction, registers::{R16Mem, R16, R8}
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Instruction {
    Nop,
    LdAA,
    Halt,
    LdR16Imm16 { reg: R16 },
    LdR16memA { reg: R16Mem },
    LdAR16mem { reg: R16Mem },
    LdImm16Sp,
    IncR16 { reg: R16 },
    DecR16 { reg: R16 },
    AddHlR16 { reg: R16 },
    IncR8 { reg: R8 },
    IncHl,
    DecR8 { reg: R8 },
    DecHl,
    LdR8Imm8 { reg: R8 },
    LdHlImm8,
    Rlca,
    Rrca,
    Rla,
    Rra,
    Daa,
    Cpl,
    Scf,
    Ccf,
}

impl TryFrom<u8> for Instruction {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let instruction = ByteInstruction(value);
        Ok(match value {
            0b00000000 => Instruction::Nop,
            0b01111111 => Instruction::LdAA,
            0b01110110 => Instruction::Halt,

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
            0b00001000 => Instruction::LdImm16Sp,

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

            // inc [hl]
            0b00110100 => Instruction::IncHl,
            // dec [hl]
            0b00110101 => Instruction::DecHl,

            // inc r8
            opcode_match!(00___100) => Instruction::IncR8 {
                reg: instruction.y().try_into()?,
            },
            // dec r8
            opcode_match!(00___101) => Instruction::DecR8 {
                reg: instruction.y().try_into()?,
            },

            // ld [hl], imm8
            0b00110110 => Instruction::LdHlImm8,
            // ld r8, imm8
            opcode_match!(00___110) => {
                Instruction::LdR8Imm8 { reg: instruction.y().try_into()? }
            }

            // rlca
            0b00000111 => Instruction::Rlca,

            // rrca
            0b00001111 => Instruction::Rrca,

            // rla
            0b00010111 => Instruction::Rla,

            // rra
            0b00011111 => Instruction::Rra,

            // daa
            0b00100111 => Instruction::Daa,

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
            Instruction::LdAA => 1,
            Instruction::Halt => 1,
            Instruction::LdR16Imm16 { .. } => 3,
            Instruction::LdR16memA { .. } => 2,
            Instruction::LdAR16mem { .. } => 2,
            Instruction::LdImm16Sp => 5,
            Instruction::IncR16 { .. } => 2,
            Instruction::DecR16 { .. } => 2,
            Instruction::AddHlR16 { .. } => 2,
            Instruction::IncR8 { .. } => 1,
            Instruction::IncHl => 3,
            Instruction::DecR8 { .. } => 1,
            Instruction::DecHl => 3,
            Instruction::LdR8Imm8 { .. } => 2,
            Instruction::LdHlImm8 => 3,
            Instruction::Rlca => 1,
            Instruction::Rrca => 1,
            Instruction::Rla => 1,
            Instruction::Rra => 1,
            Instruction::Daa => 1,
            Instruction::Cpl => 1,
            Instruction::Scf => 1,
            Instruction::Ccf => 1,
        }
    }
}
