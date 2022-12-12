use resql::core::{InputBuffer, MetaCommand, Statement, Table};

fn main() {
    let mut table = Table::new();

    loop {
        resql::print_prompt();
        let mut input_buffer = InputBuffer::new();
        input_buffer.read();

        let Some(input) = input_buffer.get_input() else {
            continue;
        };
        if input.is_empty() { // guarantee that unwrap() is ok
            continue;
        }

        match input.chars().nth(0).unwrap() { // never empty, thus ok
            '.' => MetaCommand::process(input),
            _ => Statement::prepare(input).execute(&mut table)
        }
    }
}
