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
                Self::exit(0);
            }
            MetaCommand::BTree => table.pager.get_page(0).print_leaf_node(),
            MetaCommand::Constants => Table::print_constants(),
            MetaCommand::Error(s) => crate::error(s),
        }
    }

    fn exit(code: i32) {
        println!("exitting...");
        std::process::exit(code);
    }
}
