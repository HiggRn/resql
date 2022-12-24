use crate::backend::{row, Row, Table};

pub enum Statement {
    Insert(Row),
    Select,
    Error(String),
}

impl Statement {
    pub fn prepare(input: &str) -> Statement {
        let mut args: Vec<&str> = input.split_ascii_whitespace().collect();

        let command = args.remove(0);

        match command {
            "insert" => Self::parse(&args),
            "select" => Statement::Select,
            _ => Statement::Error(format!("unknown command: '{command}'")),
        }
    }

    pub fn execute(&self, table: &mut Table) {
        match &self {
            Statement::Insert(args) => table.insert(args),
            Statement::Select => table.select(),
            Statement::Error(s) => eprintln!("[ERROR]{s}"),
        }
    }

    fn parse(args: &Vec<&str>) -> Statement {
        if args.len() < 3 {
            return Statement::Error("syntax error".into());
        }

        let Some(id) = args[0].parse().ok() else {
            return Statement::Error(
                format!("can't parse '{}' to u32", args[0])
            );
        };

        if args[1].len() > row::MAX_USERNAME {
            return Statement::Error(format!("'{}' is too long for username", args[1]));
        }

        if args[2].len() > row::MAX_EMAIL {
            return Statement::Error(format!("'{}' is too long for email", args[2]));
        }

        Statement::Insert(Row {
            id,
            username: args[1].into(),
            email: args[2].into(),
        })
    }
}
