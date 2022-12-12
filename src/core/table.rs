use super::Row;

struct Page(Vec<u8>);
impl Page {
    const PAGE_SIZE: usize = 4096;
    const MAX_ROWS: usize = Page::PAGE_SIZE / Row::ROW_SIZE;
}


pub struct Table {
    num_rows: usize,
    pages: Vec<Option<Page>>
}

impl Table {
    const MAX_PAGES: usize = 100;
    const MAX_ROWS: usize = Self::MAX_PAGES * Page::MAX_ROWS;

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
    
    pub fn insert(&mut self, row: &Row) {
        if self.num_rows == Table::MAX_ROWS {
            Self::error("table is full");
            return;
        }

        let (page, pos) = self.row_slot(self.num_rows);
        let bytes = row.serialize();
        page.0[pos..pos + bytes.len()].copy_from_slice(&bytes);
        self.num_rows += 1;
    }

    pub fn select(&mut self) {
        for row_num in 0..self.num_rows {
            let (buf, pos) = self.row_slot(row_num);
            let row = Row::deserialize(&buf.0, pos);
            println!("{}: {} {}", row.id, row.username.trim_matches('\0'), row.email.trim_matches('\0'));
        }
    }

    fn error(s: &str) {
        eprintln!("[ERROR]{s}");
    }

    fn row_slot(&mut self, row_num: usize) -> (&mut Page, usize) {
        let page_num = row_num / Page::MAX_ROWS;
        let pos = row_num % Page::MAX_ROWS;
        
        if let None = self.pages[page_num] { // make sure unwrap() won't panic
            self.pages[page_num] = Some(Page{ 0: vec![0; Page::PAGE_SIZE] });
        }
        (self.pages[page_num].as_mut().unwrap(), pos * Row::ROW_SIZE)
    }
}
