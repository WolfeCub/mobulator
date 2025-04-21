use std::ffi::OsString;

use clap::{Parser, arg, command};
use mobulator::{cpu::{Cpu, Status}, registers::R8};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

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
    assert_eq!(cpu.registers.flags(), state.f, "Flags");
    assert_eq!(cpu.registers.b(), state.b, "Register b");
    assert_eq!(cpu.registers.c(), state.c, "Register c");
    assert_eq!(cpu.registers.d(), state.d, "Register d");
    assert_eq!(cpu.registers.e(), state.e, "Register e");
    assert_eq!(cpu.registers.h(), state.h, "Register h");
    assert_eq!(cpu.registers.l(), state.l, "Register l");
    assert_eq!(cpu.registers.pc, state.pc, "Register pc");
    assert_eq!(cpu.registers.sp, state.sp, "Register sp");

    for [addr, val] in state.ram.iter() {
        assert_eq!(
            cpu.memory.get_byte(*addr).expect("Out of bounds"),
            u8::try_from(*val).expect("Unable to convert")
        );
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    test_path: String,

    #[arg(long)]
    file: Option<OsString>,

    #[arg(long, default_value_t = tracing::Level::INFO)]
    level: tracing::Level,
}

fn main() {
    let args = Args::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.level)
        .init();

    let mut files = std::fs::read_dir(args.test_path)
        .expect("Unable to read dir")
        .map(|f| f.expect("Unable to access file"))
        .filter_map(|f| {
            if let Some(filter) = &args.file {
                if f.file_name() == *filter {
                    Some(f)
                } else {
                    None
                }
            } else {
                Some(f)
            }
        })
        .collect::<Vec<_>>();
    files.sort_by_key(|file| file.path());

    'file: for file in files {
        info!("Running file '{}'", file.file_name().into_string().unwrap());
        let content = std::fs::read_to_string(file.path()).expect("Unable to read file");
        let json = serde_json::from_str::<Vec<Test>>(&content).expect("Unable to deserialize");

        for test in json.into_iter() {
            let name = test.name.clone();
            let mut cpu = setup(&test);

            debug!("  Running test '{}'", test.name);

            let mut cycles = test.cycles.len();
            while cycles > 0 {
                match cpu.run_next_instruction() {
                    Ok(Status::Cycles(c)) => {
                        cycles -= usize::from(c);
                    },
                    Err(e) => {
                        error!("Failed test '{}': {}", name, e);
                        continue 'file;
                    }
                    _ => {},
                }
            }

            verify(&test, &cpu);
        }
    }
}
