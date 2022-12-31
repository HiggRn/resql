use super::{page, row, Cursor, Pager, Row};

pub struct Table {
    pub root_page_num: usize,
    pub pager: Pager,
}

impl Table {
    pub fn open(filename: &str) -> Self {
        let mut pager = Pager::new(filename);
        if pager.get_num_pages() == 0 {
            // New database file
            pager.get_page(0).init_leaf_node();
        }

        Self {
            root_page_num: 0,
            pager,
        }
    }

    pub fn close(&mut self) {
        let num_pages = self.pager.get_num_pages();
        for page_num in 0..num_pages {
            self.pager.flush(page_num);
        }
    }

    pub fn insert(&mut self, row: &Row) {
        let page = self.pager.get_page(self.root_page_num);
        let num_cells = page.get_leaf_node_num_cells();
        if  num_cells as usize >= page::LEAF_MAX_CELLS {
            crate::error("table is full");
            return;
        }

        let key_to_insert = row.id;
        let cell_num = page.find(key_to_insert);

        if cell_num < num_cells {
            let key_at_index = page.get_leaf_node_key(cell_num);
            if key_at_index == key_to_insert {
                crate::error(format!("duplicate key '{key_to_insert}'").as_str())
            }
        }

        let mut cursor = Cursor::from_pos(self, self.root_page_num, cell_num);
        cursor.leaf_node_insert(key_to_insert, row);
    }

    pub fn select(&mut self) {
        let mut cursor = Cursor::from_start(self);

        while !cursor.end_of_table {
            let row = cursor.get_row();
            println!(
                "{}: {} {}",
                row.id,
                row.username.trim_matches('\0'),
                row.email.trim_matches('\0')
            );
            cursor.advance();
        }
    }

    pub fn print_constants() {
        println!("Constants:\n");
        println!("ROW_SIZE: {}", row::ROW_SIZE);
        println!("COMMON_HEADER_SIZE: {}", page::COMMON_HEADER_SIZE);
        println!("LEAF_HEADER_SIZE: {}", page::LEAF_HEADER_SIZE);
        println!("LEAF_CELL_SIZE: {}", page::LEAF_CELL_SIZE);
        println!("LEAF_MAX_CELLS: {}", page::LEAF_MAX_CELLS);
    }
}
