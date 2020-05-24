use std::env;
use std::thread::sleep;
use std::time::Duration;
use std::time::SystemTime;

use easycurses::constants::acs;
use easycurses::Color::*;
use easycurses::ColorPair;
use easycurses::*;
use getopts::Options;

use chip_8::Chip8;

const CYCLES_PER_SECOND: u32 = 500;
const TICKS_PER_CYCLE: u32 = (1000.0 / CYCLES_PER_SECOND as f64) as u32;
const ESC: Input = Input::Character(27 as char);

const KEY_MAP: [Input; 16] = [
    Input::Character('x'),
    Input::Character('1'),
    Input::Character('2'),
    Input::Character('3'),
    Input::Character('q'),
    Input::Character('w'),
    Input::Character('e'),
    Input::Character('a'),
    Input::Character('s'),
    Input::Character('d'),
    Input::Character('z'),
    Input::Character('c'),
    Input::Character('4'),
    Input::Character('r'),
    Input::Character('f'),
    Input::Character('v'),
];

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();

    opts.optflag("d", "debug", "display debug info");
    opts.optflag("h", "help", "display this help message");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    if matches.opt_present("h") {
        print_usage(opts);
        return;
    }

    let debug = matches.opt_present("d");

    let input = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        eprintln!("no ROM file given");
        print_usage(opts);
        return;
    };

    println!("Loading {}...", input);

    let mut chip8 = Chip8::new();

    chip8.load_program(&args[1]).unwrap();

    let mut screen = setup_screen();
    let (x_offset, y_offset) = get_offsets(&screen);

    run_loop(&mut chip8, &mut screen, x_offset, y_offset, debug);
}

fn print_usage(opts: Options) {
    let brief = format!("Usage: chip-8 [options] ROM");

    println!("{}", opts.usage(&brief));
}

fn run_loop(chip8: &mut Chip8, screen: &mut EasyCurses, x_offset: i32, y_offset: i32, debug: bool) {
    let mut iteration: u32 = 0;

    loop {
        let start = SystemTime::now();

        chip8.execute_cycle();

        if !process_input(chip8, screen) {
            break;
        }

        if chip8.draw_flag {
            draw_graphics(chip8, screen, x_offset, y_offset, iteration, debug);
        }

        let elapsed = match start.elapsed() {
            Ok(e) => e.as_millis(),
            Err(e) => panic!("time error: {}", e),
        };

        if elapsed < TICKS_PER_CYCLE as u128 {
            let time_left = TICKS_PER_CYCLE as u128 - elapsed;

            sleep(Duration::from_millis(time_left as u64));
        }

        if chip8.sound_timer > 0 {
            screen.beep();
        }

        iteration += 1;
    }
}

fn process_input(chip8: &mut Chip8, screen: &mut EasyCurses) -> bool {
    if let Some(key) = screen.get_input() {
        return if key == ESC {
            false // exit on `Esc`
        } else {
            for i in 0..16 as usize {
                if key == KEY_MAP[i] {
                    chip8.key[i] = 1;
                }
            }

            true
        };
    }

    return true;
}

fn get_offsets(screen: &EasyCurses) -> (i32, i32) {
    let (rows, cols) = screen.get_row_col_count();

    (rows / 2 - 16, cols / 2 - 32)
}

fn draw_graphics(
    chip8: &mut Chip8,
    screen: &mut EasyCurses,
    x_offset: i32,
    y_offset: i32,
    iteration: u32,
    debug: bool,
) {
    let rows = 32;
    let cols = 64;

    chip8.draw_flag = false;

    if debug {
        screen.move_rc(0 + x_offset - 1, 0 + y_offset);
        screen.print(format!("Iteration: {}", iteration));
    }

    screen.move_rc(0 + x_offset, 0 + y_offset);
    screen.print_char(acs::ulcorner());

    for i in 0..=cols {
        screen.move_rc(0 + x_offset, i + 1 + y_offset);
        screen.print_char(acs::hline());
    }

    screen.move_rc(0 + x_offset, cols + 1 + y_offset);
    screen.print_char(acs::urcorner());

    for r in 0..rows {
        screen.move_rc(r + 1 + x_offset, 0 + y_offset);
        screen.print_char(acs::vline());

        for c in 0..cols {
            let pixel = if chip8.gfx[(c + r * cols) as usize] == 1 {
                '*'
            } else {
                ' '
            };

            screen.move_rc(r + 1 + x_offset, c + 1 + y_offset);
            screen.print_char(pixel);
        }

        screen.move_rc(r + 1 + x_offset, cols + 1 + y_offset);
        screen.print_char(acs::vline());
    }

    screen.move_rc(rows + 1 + x_offset, 0 + y_offset);
    screen.print_char(acs::llcorner());

    for i in 0..=cols {
        screen.move_rc(rows + 1 + x_offset, i + 1 + y_offset);
        screen.print_char(acs::hline());
    }

    screen.move_rc(rows + 1 + x_offset, cols + 1 + y_offset);
    screen.print_char(acs::lrcorner());

    screen.refresh();
}

fn setup_screen() -> EasyCurses {
    let mut screen = EasyCurses::initialize_system().unwrap();

    screen.set_cursor_visibility(CursorVisibility::Invisible);
    screen.set_echo(false);
    screen.set_color_pair(colorpair!(White on Black));
    screen.set_input_mode(InputMode::Character);
    screen.set_input_timeout(TimeoutMode::Immediate);
    screen
}
