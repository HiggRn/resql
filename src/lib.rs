use std::io::{self, Write};

pub mod core;

pub fn print_prompt() {
    print!(">> ");
    io::stdout()
        .flush()
        .expect("error: stdout flush");
}
