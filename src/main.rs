use std::env;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

use easycurses::constants::acs;
use easycurses::Color::*;
use easycurses::ColorPair;
use easycurses::*;

use chip_8::Chip8;

static KEY_MAP: [Input; 16] = [
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

    if args.len() == 1 {
        println!("Usage: chip-8 <filename>");
        exit(1);
    }

    println!("Loading {}...", &args[1]);

    let mut chip8 = Chip8::new();

    chip8.load_program(&args[1]).unwrap();

    let mut screen = setup_screen();
    let (x_offset, y_offset) = get_offsets(&screen);

    run_loop(&mut chip8, &mut screen, x_offset, y_offset);
}

pub fn run_loop(chip8: &mut Chip8, screen: &mut EasyCurses, x_offset: i32, y_offset: i32) {
    let mut iteration: u32 = 0;

    loop {
        chip8.execute_cycle();

        if !process_input(chip8, screen) {
            break;
        }

        if chip8.draw_flag {
            draw_graphics(chip8, screen, x_offset, y_offset, iteration);
        }

        sleep(Duration::from_micros(1200));
        iteration += 1;
    }
}

fn process_input(chip8: &mut Chip8, screen: &mut EasyCurses) -> bool {
    chip8.clear_keys();

    if let Some(key) = screen.get_input() {
        return if key == Input::Character(27 as char) {
            false // exit on `Esc`
        } else {
            for i in 0..16 {
                if key == KEY_MAP[i as usize] {
                    chip8.key[i as usize] = 1;
                    break;
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
) {
    let rows = 32;
    let cols = 64;

    chip8.draw_flag = false;

    screen.move_rc(0 + x_offset - 1, 0 + y_offset);
    screen.print(format!("Iteration: {}", iteration));

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

pub fn setup_screen() -> EasyCurses {
    let mut screen = EasyCurses::initialize_system().unwrap();

    screen.set_cursor_visibility(CursorVisibility::Invisible);
    screen.set_echo(false);
    screen.set_color_pair(colorpair!(White on Black));
    screen.set_input_mode(InputMode::Character);
    screen.set_input_timeout(TimeoutMode::Immediate);
    screen
}
