use super::{page::{self, NodeType}, row, Cursor, Pager, Row};

pub struct Table {
    pub root_page_num: usize,
    pub pager: Pager,
}

impl Table {
    pub fn open(filename: &str) -> Self {
        let mut pager = Pager::new(filename);
        if pager.num_pages == 0 {
            // New database file
            let page = pager.get_page(0);
            page.init_leaf();
            page.set_is_root(true);
            pager.num_pages = 1;
        }

        Self {
            root_page_num: 0,
            pager,
        }
    }

    pub fn close(&mut self) {
        let num_pages = self.pager.num_pages;
        for page_num in 0..num_pages {
            self.pager.flush(page_num);
        }
    }

    pub fn insert(&mut self, row: &Row) {
        let key_to_insert = row.id as usize;

        let (page_num, cell_num) = self.find(key_to_insert, self.root_page_num);

        let page = self.pager.get_page(page_num);
        let num_cells = page.get_leaf_num_cells();
        if cell_num < num_cells {
            let key_at_index = page.get_leaf_key(cell_num);
            if key_at_index == key_to_insert {
                crate::error(format!("duplicate key '{key_to_insert}'").as_str());
                return;
            }
        }

        let mut cursor = Cursor::from_pos(self, page_num, cell_num);
        cursor.leaf_insert(key_to_insert, row);
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

    pub fn new_root(&mut self, right_child_page_num: usize) {
        let root_copy = self.pager.copy_page(self.root_page_num).unwrap();
        let left_child_page_num = self.pager.get_unused_page_num();
        let left_child = self.pager.get_page(left_child_page_num);
        left_child.clone_from(&root_copy);
        left_child.set_is_root(false);
        left_child.set_parent(self.root_page_num);
        let left_child_max_key = left_child.get_max_key();
        self.pager.num_pages += 1;

        let right_child = self.pager.get_page(right_child_page_num);
        right_child.set_parent(self.root_page_num);

        let root = self.pager.get_page(self.root_page_num);
        root.init_internal();
        root.set_is_root(true);
        root.set_internal_num_keys(1);
        root.set_internal_child(0, left_child_page_num);
        root.set_internal_key(0, left_child_max_key);
        root.set_internal_right_child(right_child_page_num);
    }

    pub fn print_constants() {
        println!("Constants:\n");
        println!("ROW_SIZE: {}", row::ROW_SIZE);
        println!("COMMON_HEADER_SIZE: {}", page::COMMON_HEADER_SIZE);
        println!("LEAF_HEADER_SIZE: {}", page::LEAF_HEADER_SIZE);
        println!("LEAF_CELL_SIZE: {}", page::LEAF_CELL_SIZE);
        println!("LEAF_MAX_CELLS: {}", page::LEAF_MAX_CELLS);
    }

    pub fn find(&mut self, key: usize, start_page_num: usize) -> (usize, usize) {
        let start_page = self.pager.get_page(start_page_num);
        
        match start_page.get_type() {
            NodeType::Leaf => (start_page_num, start_page.leaf_find(key)),
            NodeType::Internal => {
                let child_num = start_page.internal_find(key);
                let child_page_num = start_page.get_internal_child(child_num);
                self.find(key, child_page_num)
            },
        }
    }

    pub fn internal_insert(&mut self, parent_page_num: usize, child_page_num: usize) {
        let child = self.pager.get_page(child_page_num);
        let child_max_key = child.get_max_key();
        
        let parent = self.pager.get_page(parent_page_num);
        let index = parent.internal_find(child_max_key);
        
        let original_num_keys = parent.get_internal_num_keys();
        parent.set_internal_num_keys(original_num_keys + 1);

        if original_num_keys > page::INTERNAL_MAX_CELLS {
            unimplemented!("need to implement splitting internal node");
        }

        let right_child_page_num = parent.get_internal_right_child();
        let right_child = self.pager.get_page(right_child_page_num);
        let right_child_max_key = right_child.get_max_key();

        let parent = self.pager.get_page(parent_page_num); // the same as 'parent' above
        if child_max_key > right_child_max_key {
            // Replace right child
            parent.set_internal_child(original_num_keys, right_child_page_num);
            parent.set_internal_key(original_num_keys, right_child_max_key);
            parent.set_internal_right_child(child_page_num);
        } else {
            // Make room for the new cell
            for i in (index + 1..=original_num_keys).rev() {
                let key = parent.get_internal_key(i);
                let child = parent.get_internal_child(i);
                parent.set_internal_key(i - 1, key);
                parent.set_internal_child(i - 1, child);
            }
            parent.set_internal_child(index, child_page_num);
            parent.set_internal_key(index, child_max_key);
        }
    }
}
