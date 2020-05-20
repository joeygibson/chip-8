use std::env;
use std::process::exit;
use chip_8::Chip8;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("Usage: chip-8 <filename>");
        exit(1);
    }

    println!("Loading {}...", args[1]);

    let chip_8 = Chip8::new();

    chip_8.
}
