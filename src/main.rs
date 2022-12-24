use std::env;

use resql::backend::Table;
use resql::core::{InputBuffer, MetaCommand, Statement};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("error: must supply a database filename.");
        std::process::exit(0x0100);
    }
    let mut table = Table::open(args[1].as_str());

    loop {
        resql::print_prompt();
        let mut input_buffer = InputBuffer::new();
        input_buffer.read();

        let Some(input) = input_buffer.get_input() else {
            continue;
        };
        if input.is_empty() {
            // guarantee that unwrap() is ok
            continue;
        }

        match input.chars().next().unwrap() {
            // never empty, thus ok
            '.' => MetaCommand::process(input, &mut table),
            _ => Statement::prepare(input).execute(&mut table),
        }
    }
}
