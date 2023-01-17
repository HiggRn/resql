use crate::backend::Table;

pub enum MetaCommand {
    Exit,
    BTree,
    Constants,
    Error(String),
}

impl MetaCommand {
    pub fn process(input: &str, table: &mut Table) {
        let words: Vec<&str> = input.split_ascii_whitespace().collect();
        let result = match words[0] {
            ".exit" => Self::Exit,
            ".btree" => Self::BTree,
            ".constants" => Self::Constants,
            _ => Self::Error(format!("unknown metacommand: '{}'", words[0])),
        };

        match &result {
            MetaCommand::Exit => {
                table.close();
                println!("exitting...");
                std::process::exit(0);
            }
            MetaCommand::BTree => table.pager.print(table.root_page_num, 0),
            MetaCommand::Constants => Table::print_constants(),
            MetaCommand::Error(s) => crate::error(s),
        }
    }
}
