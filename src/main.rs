use std::env;
use std::process::exit;
use chip_8::Chip8;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("Usage: chip-8 <filename>");
        exit(1);
    }

    println!("Loading {}...", &args[1]);

    let mut chip8 = Chip8::new();

    chip8.load_program(&args[1]).unwrap();

    chip8.execute_cycle();
}
