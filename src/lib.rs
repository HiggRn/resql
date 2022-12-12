use std::io::{self, Write};

pub mod core;
pub mod backend;

pub fn print_prompt() {
    print!(">> ");
    io::stdout()
        .flush()
        .expect("error: stdout flush");
}

fn error(s: &str) {
    eprintln!("[ERROR]{s}");
}
