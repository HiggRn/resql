use std::cmp;
use std::io::{self, Write};
use std::mem;
use std::ops::{Index, IndexMut, Range};

use byteorder::{LittleEndian, ByteOrder};

struct InputBuffer {
    buffer: String,
    input_length: usize
}

impl InputBuffer {
    pub fn new() -> InputBuffer {
        InputBuffer {
            buffer: String::new(),
            input_length: 0
        }
    }

    pub fn read(&mut self) {
        io::stdin()
            .read_line(&mut self.buffer)
            .expect("error: stdin read_line");
        if !self.buffer.is_empty() {
            self.input_length = self.buffer.len();
        }
    }

    pub fn get_input(&self) -> Option<&str> {
        if self.buffer.is_empty() {
            None
        } else {
            Some(self.buffer.trim_end())
        }
    }
}

fn print_prompt() {
    print!(">> ");
    io::stdout()
        .flush()
        .expect("error: stdout flush");
}

enum MetaCommand {
    Exit,
    Error(String)
}

impl MetaCommand {
    pub fn process(input: &str) {
        let words: Vec<&str> = input.split_ascii_whitespace().collect();
        let result = match words[0] {
            ".exit" => Self::Exit,
            _ => Self::Error(format!("unknown metacommand: '{}'", words[0]))
        };

        match &result {
            MetaCommand::Exit => Self::exit(0),
            MetaCommand::Error(s) => eprintln!("[ERROR]{s}")
        }
    }

    fn exit(code: i32) {
        println!("exitting...");
        std::process::exit(code);
    }
}

enum Statement {
    Insert(Row),
    Select,
    Error(String)
}

impl Statement {
    pub fn prepare(input: &str) -> Statement {
        let mut args: Vec<&str> = input.split_ascii_whitespace().collect();
        
        let command = args.remove(0);

        match command {
            "insert" => Self::parse(&args),
            "select" => Statement::Select,
            _ => Statement::Error(format!("unknown command: '{command}'"))
        }
    }

    pub fn execute(&self, table: &mut Table) {
        match &self {
            Statement::Insert(args) => table.insert(args),
            Statement::Select => table.select(),
            Statement::Error(s) => eprintln!("[ERROR]{s}")
        }
    }

    fn parse(args: &Vec<&str>) -> Statement {        
        if args.len() < 3 {
            return Statement::Error(format!("syntax error"));
        }
        
        let Some(id) = args[0].parse().ok() else {
            return Statement::Error(
                format!("can't parse '{}' to u32", args[0])
            );
        };

        if args[1].len() > Row::MAX_USERNAME {
            return Statement::Error(
                format!("'{}' is too long for username", args[1])
            );
        }

        if args[2].len() > Row::MAX_EMAIL {
            return Statement::Error(
                format!("'{}' is too long for email", args[2])
            );
        }

        Statement::Insert(Row {
            id,
            username: args[1].into(),
            email: args[2].into()
        })
    }    
}

#[derive(Clone)]
struct Row {
    id: u32,
    username: String,
    email: String
}

impl Row {
    const MAX_USERNAME: usize = 31;
    const MAX_EMAIL: usize = 255;
    const ID_SIZE: usize = mem::size_of::<u32>();
    const USERNAME_SIZE: usize = Self::MAX_USERNAME + 1;
    const EMAIL_SIZE: usize = Self::MAX_EMAIL + 1;
    const USERNAME_OFFSET: usize = Self::ID_SIZE;
    const EMAIL_OFFSET: usize = Self::USERNAME_OFFSET + Self::USERNAME_SIZE;
    const ROW_SIZE: usize = Self::ID_SIZE + Self::USERNAME_SIZE + Self::EMAIL_SIZE;

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = vec![0; Self::ROW_SIZE];
        LittleEndian::write_u32(
            &mut buf.index_mut(Range {
                start: 0,
                end: Self::ID_SIZE,
            }),
            self.id,
        );
        Self::write_string(&mut buf, Self::USERNAME_OFFSET, &self.username, Self::USERNAME_SIZE);
        Self::write_string(&mut buf, Self::EMAIL_OFFSET, &self.email, Self::EMAIL_SIZE);
        return buf;
    }

    pub fn deserialize(buf: &Vec<u8>, pos: usize) -> Self {
        let mut bytes = vec![0; Self::ROW_SIZE];
        bytes.clone_from_slice(buf.index(Range {
            start: pos,
            end: pos + Self::ROW_SIZE,
        }));

        let id = LittleEndian::read_u32(&bytes[0..4]);
        let username = Self::read_string(&bytes, Self::USERNAME_OFFSET, Self::USERNAME_SIZE);
        let email = Self::read_string(&bytes, Self::EMAIL_OFFSET, Self::EMAIL_SIZE);
        Self {
            id,
            username,
            email,
        }
    }

    fn write_string(buf: &mut Vec<u8>, pos: usize, s: &str, length: usize) {
        let bytes = s.as_bytes();
        let len = bytes.len();
        buf[pos..pos + len].copy_from_slice(bytes);
        buf[pos + len..pos + length].copy_from_slice(&vec![0u8; length - len]);
    }

    fn read_string(buf: &Vec<u8>, pos: usize, length: usize) -> String { // buf.len() MUST be greater than pos
        let len = cmp::min(length, buf.len() - pos);
        let mut bytes = vec![0; len];
        bytes.clone_from_slice(buf.index(Range { start: pos, end: pos + len}));
        return String::from_utf8(bytes).unwrap();
    }
}

type Page = Vec<u8>;
const MAX_ROWS: usize = PAGE_SIZE / Row::ROW_SIZE;
const PAGE_SIZE: usize = 4096;

struct Table {
    num_rows: usize,
    pages: Vec<Option<Page>>
}

impl Table {
    const MAX_PAGES: usize = 100;
    const MAX_ROWS: usize = Table::MAX_PAGES * MAX_ROWS;

    pub fn new() -> Self {
        let mut table = Self {
            num_rows: 0,
            pages: Vec::with_capacity(Self::MAX_PAGES)
        };
        for _ in 0..Self::MAX_PAGES {
            table.pages.push(None);
        }
        table
    }

    fn error(s: &str) {
        eprintln!("[ERROR]{s}");
    }
    
    fn insert(&mut self, row: &Row) {
        if self.num_rows == Table::MAX_ROWS {
            Self::error("table is full");
            return;
        }

        let (page, pos) = self.row_slot(self.num_rows);
        let bytes = row.serialize();
        page[pos..pos + bytes.len()].copy_from_slice(&bytes);
        self.num_rows += 1;
    }

    fn select(&mut self) {
        for row_num in 0..self.num_rows {
            let (buf, pos) = self.row_slot(row_num);
            let row = Row::deserialize(&buf, pos);
            println!("{}: {} {}", row.id, row.username.trim_matches('\0'), row.email.trim_matches('\0'));
        }
    }

    fn row_slot(&mut self, row_num: usize) -> (&mut Page, usize) {
        let page_num = row_num / MAX_ROWS;
        let pos = row_num % MAX_ROWS;
        
        if let None = self.pages[page_num] { // make sure unwrap() won't panic
            self.pages[page_num] = Some(vec![0; PAGE_SIZE]);
        }
        (self.pages[page_num].as_mut().unwrap(), pos * Row::ROW_SIZE)
    }
}

fn main() {
    let mut table = Table::new();

    loop {
        print_prompt();
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