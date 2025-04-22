use std::u16;

use crate::{
    byte_instruction::ByteInstruction,
    cpu::Cpu,
    instructions::*,
    registers::{Cond, R8, R16},
};
use mobulator_macros::opcode_list;

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
    cpu.memory.load_instructions(&[LD_IMM16_SP, first, second]);

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
            cpu.set_r8(reg, 137);

            cpu.run_next_instruction()
                .expect("Unable to process CPU instructions");

            cpu.get_r8(reg).unwrap()
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

    cpu.set_r8(R8::B, 0b0000_1111);

    cpu.run_next_instruction()
        .expect("Unable to process CPU instructions");

    assert_eq!(cpu.registers.h_flg(), true);
    assert_eq!(cpu.registers.n_flg(), false);

    // dec
    let mut cpu = Cpu::default();
    cpu.memory.load_instructions(&[0b00000101]);

    cpu.set_r8(R8::B, 0b0001_0000);

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
        cpu.memory.load_instructions(&[instruction, 0b00111100]);
        cpu.registers.hl = 0xDC17;

        let instruction = ByteInstruction(instruction);

        cpu.run_next_instruction()
            .expect("Unable to process CPU instructions");

        let val = if instruction.0 != LD_HL_IMM8 {
            cpu.get_r8(instruction.y().try_into().unwrap()).unwrap()
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

#[test]
fn daa() {
    // daa
    let mut cpu = Cpu::default();
    cpu.memory.load_instructions(&[DAA]);

    cpu.registers.af = 0b00000000_10100000;
    cpu.registers.set_a(0x7c);

    cpu.run_next_instruction()
        .expect("Unable to process CPU instructions");

    assert_eq!(cpu.registers.a(), 0x82);
    assert_eq!(cpu.registers.z_flg(), false);
    assert_eq!(cpu.registers.n_flg(), false);
    assert_eq!(cpu.registers.h_flg(), false);
    assert_eq!(cpu.registers.c_flg(), false);

    let mut cpu = Cpu::default();
    cpu.memory.load_instructions(&[DAA]);

    cpu.registers.af = 0b00000000_10100000;
    cpu.registers.set_a(0x9c);

    cpu.run_next_instruction()
        .expect("Unable to process CPU instructions");

    assert_eq!(cpu.registers.a(), 0x02);
    assert_eq!(cpu.registers.z_flg(), false);
    assert_eq!(cpu.registers.n_flg(), false);
    assert_eq!(cpu.registers.h_flg(), false);
    assert_eq!(cpu.registers.c_flg(), true);
}

#[test]
fn cpl() {
    // cpl
    let mut cpu = Cpu::default();
    cpu.memory.load_instructions(&[CPL]);

    cpu.registers.af = 0b10111010_00000000;

    cpu.run_next_instruction()
        .expect("Unable to process CPU instructions");

    assert_eq!(cpu.registers.a(), 0b01000101);
    assert_eq!(cpu.registers.z_flg(), false);
    assert_eq!(cpu.registers.n_flg(), true);
    assert_eq!(cpu.registers.h_flg(), true);
    assert_eq!(cpu.registers.c_flg(), false);
}

#[test]
fn scf() {
    // scf
    let mut cpu = Cpu::default();
    cpu.memory.load_instructions(&[SCF]);

    cpu.registers.af = 0b00000000_11100000;

    cpu.run_next_instruction()
        .expect("Unable to process CPU instructions");

    assert_eq!(cpu.registers.z_flg(), true);
    assert_eq!(cpu.registers.n_flg(), false);
    assert_eq!(cpu.registers.h_flg(), false);
    assert_eq!(cpu.registers.c_flg(), true);
}

#[test]
fn ccf() {
    // ccf
    let mut cpu = Cpu::default();
    cpu.memory.load_instructions(&[CCF]);

    cpu.registers.af = 0b00000000_11110000;

    cpu.run_next_instruction()
        .expect("Unable to process CPU instructions");

    assert_eq!(cpu.registers.z_flg(), true);
    assert_eq!(cpu.registers.n_flg(), false);
    assert_eq!(cpu.registers.h_flg(), false);
    assert_eq!(cpu.registers.c_flg(), false);
}

#[test]
fn jr_imm8() {
    // jr imm8
    let mut cpu = Cpu::default();
    cpu.memory.load_instructions(&[JR_IMM8, 37]);

    cpu.run_next_instruction()
        .expect("Unable to process CPU instructions");

    assert_eq!(cpu.registers.pc, 39);
}

#[test]
fn jr_cond_imm8() {
    // jr cond, imm8
    for instruction in opcode_list!(001__000) {
        let mut cpu = Cpu::default();
        cpu.memory.load_instructions(&[instruction, 37]);
        cpu.memory.memory[39] = instruction;
        cpu.memory.memory[40] = 235;

        let cond = Cond::try_from(ByteInstruction(instruction).cond()).expect("Can't create Cond");
        match cond {
            Cond::Z => cpu.registers.set_z_flg(true),
            Cond::C => cpu.registers.set_c_flg(true),
            _ => {}
        }

        cpu.run_next_instruction()
            .expect("Unable to process CPU instructions");

        assert_eq!(cpu.registers.pc, 39);

        cpu.run_next_instruction()
            .expect("Unable to process CPU instructions");

        assert_eq!(cpu.registers.pc, 20);
    }
}

#[test]
fn ld_r8_r8() {
    // ld r8, r8
    for instruction in opcode_list!(01______) {
        if instruction == HALT {
            continue;
        }

        let mut cpu = Cpu::default();
        cpu.memory.load_instructions(&[instruction]);
        cpu.registers.hl = 50; // Out of the way of instructions

        let byte = ByteInstruction(instruction);
        let src: R8 = byte.z().try_into().unwrap();
        let dst: R8 = byte.y().try_into().unwrap();

        cpu.set_r8(dst, 22);
        cpu.set_r8(src, 37);

        cpu.run_next_instruction()
            .expect("Unable to process CPU instructions");

        assert_eq!(cpu.get_r8(dst).unwrap(), 37);
    }
}

#[test]
fn add_a_r8() {
    // add a, r8
    for set_carry in [true, false] {
        for instruction in opcode_list!(1000____) {
            let mut cpu = Cpu::default();
            cpu.memory.load_instructions(&[instruction]);
            cpu.registers.hl = 50; // Out of the way of instructions

            let byte = ByteInstruction(instruction);
            let reg = byte.z().try_into().unwrap();

            cpu.registers.set_a(73);
            cpu.set_r8(reg, 37);
            cpu.registers.set_c_flg(set_carry);


            cpu.run_next_instruction()
                .expect("Unable to process CPU instructions");

            let target = if reg == R8::A { 74 } else { 110 };
            let carry = if byte.q() && set_carry { 1 } else { 0 };
            assert_eq!(cpu.registers.a(), target + carry);
        }
    }
}

#[test]
fn sub_a_r8() {
    // sub a, r8
    for set_carry in [true, false] {
        for instruction in opcode_list!(1001____) {
            let mut cpu = Cpu::default();
            cpu.memory.load_instructions(&[instruction]);
            cpu.registers.hl = 50; // Out of the way of instructions

            let byte = ByteInstruction(instruction);
            let reg = byte.z().try_into().unwrap();

            cpu.registers.set_a(100);
            cpu.set_r8(reg, 22);
            cpu.registers.set_c_flg(set_carry);

            cpu.run_next_instruction()
                .expect("Unable to process CPU instructions");

            let target: u8 = if reg == R8::A { 0 } else { 78 };
            let carry: u8 = if byte.q() && set_carry { 1 } else { 0 };
            assert_eq!(cpu.registers.a(), target.wrapping_sub(carry));
        }
    }
}

#[test]
fn and_a_r8() {
    // and a, r8
    for instruction in opcode_list!(10100___) {
        let mut cpu = Cpu::default();
        cpu.memory.load_instructions(&[instruction]);
        cpu.registers.hl = 50; // Out of the way of instructions

        let byte = ByteInstruction(instruction);
        let reg = byte.z().try_into().unwrap();

        cpu.registers.set_a(100);
        cpu.set_r8(reg, 22);

        cpu.run_next_instruction()
            .expect("Unable to process CPU instructions");

        let target: u8 = if reg == R8::A { 22 } else { 4 };
        assert_eq!(cpu.registers.a(), target);
    }
}

#[test]
fn xor_a_r8() {
    // xor a, r8
    for instruction in opcode_list!(10101___) {
        let mut cpu = Cpu::default();
        cpu.memory.load_instructions(&[instruction]);
        cpu.registers.hl = 50; // Out of the way of instructions

        let byte = ByteInstruction(instruction);
        let reg = byte.z().try_into().unwrap();

        cpu.registers.set_a(100);
        cpu.set_r8(reg, 22);

        cpu.run_next_instruction()
            .expect("Unable to process CPU instructions");

        let target: u8 = if reg == R8::A { 0 } else { 114 };
        assert_eq!(cpu.registers.a(), target);
    }
}

#[test]
fn or_a_r8() {
    // or a, r8
    for instruction in opcode_list!(10110___) {
        let mut cpu = Cpu::default();
        cpu.memory.load_instructions(&[instruction]);
        cpu.registers.hl = 50; // Out of the way of instructions

        let byte = ByteInstruction(instruction);
        let reg = byte.z().try_into().unwrap();

        cpu.registers.set_a(100);
        cpu.set_r8(reg, 22);

        cpu.run_next_instruction()
            .expect("Unable to process CPU instructions");

        let target: u8 = if reg == R8::A { 22 } else { 118 };
        assert_eq!(cpu.registers.a(), target);
    }
}
