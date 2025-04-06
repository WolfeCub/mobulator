use mobulator_macros::{opcode_list, opcode_match};

fn main() {
    let instruction = 1;
    match instruction {
        opcode_match!(00__0001) => {
            println!("It matches");
        }
        _ => {
            println!("naw b");
        }
    }
}
