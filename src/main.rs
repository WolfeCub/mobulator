use mobulator_macros::instructions;

fn main() {
    let instruction = 50;
    match instruction {
        instructions!(00__0010) => {
            println!("It matches");
        }
        _ => {
            println!("naw b");
        }
    }
}
