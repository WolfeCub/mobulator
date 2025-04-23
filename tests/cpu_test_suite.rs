use mobulator::cpu::{Cpu, Status};
use mobulator_test_macros::gen_test;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Test {
    name: String,
    #[serde(rename = "initial")]
    initial_state: State,
    #[serde(rename = "final")]
    final_state: State,
    cycles: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct State {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    pc: u16,
    sp: u16,
    ram: Vec<[u16; 2]>,
}

fn setup(test: &Test) -> Cpu {
    let mut cpu = Cpu::default();

    let state = &test.initial_state;
    cpu.registers.af = u16::from_be_bytes([state.a, state.f]);
    cpu.registers.bc = u16::from_be_bytes([state.b, state.c]);
    cpu.registers.de = u16::from_be_bytes([state.d, state.e]);
    cpu.registers.hl = u16::from_be_bytes([state.h, state.l]);
    cpu.registers.pc = state.pc;
    cpu.registers.sp = state.sp;

    for [addr, val] in state.ram.iter() {
        cpu.memory
            .set_byte(*addr, u8::try_from(*val).expect("Unable to convert"));
    }

    cpu
}

fn verify(test: &Test, cpu: &Cpu) {
    let state = &test.final_state;

    assert_eq!(cpu.registers.a(), state.a, "Register a");
    assert_eq!(cpu.registers.b(), state.b, "Register b");
    assert_eq!(cpu.registers.c(), state.c, "Register c");
    assert_eq!(cpu.registers.d(), state.d, "Register d");
    assert_eq!(cpu.registers.e(), state.e, "Register e");
    assert_eq!(cpu.registers.h(), state.h, "Register h");
    assert_eq!(cpu.registers.l(), state.l, "Register l");
    assert_eq!(cpu.registers.pc, state.pc, "Register pc");
    assert_eq!(cpu.registers.sp, state.sp, "Register sp");
    assert_eq!(
        cpu.registers.flags(),
        state.f,
        "Flags {:08b} vs {:08b}",
        cpu.registers.flags(),
        state.f
    );

    for [addr, val] in state.ram.iter() {
        assert_eq!(
            cpu.memory.get_byte(*addr).expect("Out of bounds"),
            u8::try_from(*val).expect("Unable to convert")
        );
    }
}

fn load_file(path: String) -> Vec<Test> {
    let content = std::fs::read_to_string(path).expect("Unable to read file");
    serde_json::from_str::<Vec<Test>>(&content).expect("Unable to deserialize")
}

fn test_file(tests: Vec<Test>) {
    for test in tests {
        let name = test.name.clone();
        let mut cpu = setup(&test);

        println!("Running test '{}'", test.name);

        let mut cycles = test.cycles.len();
        while cycles > 0 {
            match cpu.run_next_instruction() {
                Ok(Status::Cycles(c)) => {
                    cycles -= usize::from(c);
                }
                Err(e) => {
                    panic!("Failed test '{}': {}", name, e);
                }
                _ => {}
            }
        }

        verify(&test, &cpu);
    }
}

gen_test!(0x00, 0x0F);
// 0x10 STOP
gen_test!(0x11, 0x75);
// 0x76 HALT
gen_test!(0x77, 0xBF);

// TODO: Merge these single instructions when gaps are implemented

// ret cond
// ret
// ret i
gen_test!(0xC0);
gen_test!(0xD0);
gen_test!(0xC8, 0xC9);
gen_test!(0xD8, 0xD9);

// jp cond, imm16
gen_test!(0xC2);
