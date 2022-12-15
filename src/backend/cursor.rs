use super::{Page, Row, Table};

pub struct Cursor<'a> {
    table: &'a mut Table,
    row_num: usize,
    pub end_of_table: bool
}

impl Cursor<'_> {
    pub fn from_start(table: &mut Table) -> Cursor {
        Cursor {
            row_num: 0,
            end_of_table: table.num_rows == 0,
            table
        }
    }

    pub fn from_end(table: &mut Table) -> Cursor {
        Cursor {
            row_num: table.num_rows,
            end_of_table: true,
            table
        }
    }

    pub fn get_value(&mut self) -> (&mut Page, usize) {
        let page_num = self.row_num / Page::MAX_ROWS;
        let pos = self.row_num % Page::MAX_ROWS;

        (self.table.pager.get_page(page_num), pos * Row::ROW_SIZE)
    }

    pub fn advance(&mut self) {
        self.row_num += 1;
        if self.row_num == self.table.num_rows {
            self.end_of_table = true;
        }
    }
}
