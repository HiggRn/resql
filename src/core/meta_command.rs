use crate::backend::Table;

pub enum MetaCommand {
    Exit,
    Error(String),
}

impl MetaCommand {
    pub fn process(input: &str, table: &mut Table) {
        let words: Vec<&str> = input.split_ascii_whitespace().collect();
        let result = match words[0] {
            ".exit" => Self::Exit,
            _ => Self::Error(format!("unknown metacommand: '{}'", words[0])),
        };

        match &result {
            MetaCommand::Exit => Self::exit(0, table),
            MetaCommand::Error(s) => eprintln!("[ERROR]{s}"),
        }
    }

    fn exit(code: i32, table: &mut Table) {
        table.close();
        println!("exitting...");
        std::process::exit(code);
    }
}
