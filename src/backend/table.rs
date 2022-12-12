use super::{Page, Pager, Row};

pub struct Table {
    num_rows: usize,
    pager: Pager
}

impl Table {    
    const MAX_ROWS: usize = Pager::MAX_PAGES * Page::MAX_ROWS;

    pub fn open(filename: &str) -> Self {
        let pager = Pager::new(filename);
        let num_rows = pager.get_num_rows();

        Self {
            num_rows,
            pager
        }
    }

    pub fn close(&mut self) {
        let num_full_pages = self.num_rows / Page::MAX_ROWS;
        for page_num in 0..num_full_pages {
            self.pager.flush(page_num, Page::PAGE_SIZE);
        }

        // There may be a partial page to write to the end of the file
        // This should not be needed after we switch to a B-tree
        let num_rows = self.num_rows % Page::MAX_ROWS;
        if num_rows > 0 {
            self.pager.flush(num_full_pages, num_rows * Row::ROW_SIZE);
        }
    }
    
    pub fn insert(&mut self, row: &Row) {
        if self.num_rows == Table::MAX_ROWS {
            crate::error("table is full");
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

    fn row_slot(&mut self, row_num: usize) -> (&mut Page, usize) {
        let page_num = row_num / Page::MAX_ROWS;
        let pos = row_num % Page::MAX_ROWS;
        (self.pager.get_page(page_num), pos * Row::ROW_SIZE)
    }
}
