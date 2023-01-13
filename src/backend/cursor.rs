use std::cmp::Ordering;

use super::{page, Row, Table};

pub struct Cursor<'a> {
    table: &'a mut Table,
    page_num: usize,
    pub cell_num: usize,
    pub end_of_table: bool,
}

impl<'a> Cursor<'a> {
    pub fn from_start(table: &'a mut Table) -> Self {
        let num_cells = table
            .pager
            .get_page(table.root_page_num)
            .get_leaf_num_cells();

        Self {
            page_num: table.root_page_num,
            cell_num: 0,
            end_of_table: num_cells == 0,
            table,
        }
    }

    pub fn from_pos(table: &'a mut Table, page_num: usize, cell_num: usize) -> Cursor {
        let is_last_page = page_num == table.pager.num_pages;
        let page = table.pager.get_page(page_num);
        let is_last_cell = page.get_leaf_num_cells() == cell_num;

        Cursor {
            table,
            page_num,
            cell_num,
            end_of_table: is_last_page && is_last_cell,
        }
    }

    pub fn get_row(&mut self) -> Row {
        let page_num = self.page_num;
        let page = self.table.pager.get_page(page_num);

        Row::deserialize(&page.get_leaf_value(self.cell_num))
    }

    pub fn advance(&mut self) {
        let page_num = self.page_num;
        let page = self.table.pager.get_page(page_num);
        self.cell_num += 1;
        if self.cell_num >= page.get_leaf_num_cells() {
            self.end_of_table = true;
        }
    }

    pub fn leaf_insert(&mut self, key: usize, value: &Row) {
        let page = self.table.pager.get_page(self.page_num);
        let num_cells = page.get_leaf_num_cells();
        if num_cells >= page::LEAF_MAX_CELLS {
            // Node full
            self.leaf_split_and_insert(key, value);
            return;
        }

        if self.cell_num < num_cells {
            // Make room for new cell
            for i in (self.cell_num..num_cells).rev() {
                let cell = page.get_leaf_cell(i);
                page.set_leaf_cell(i + 1, cell);
            }
        }

        page.set_leaf_num_cells(num_cells + 1);
        page.set_leaf_key(self.cell_num, key);
        page.set_leaf_value(self.cell_num, value.serialize());
    }

    pub fn leaf_split_and_insert(&mut self, key: usize, value: &Row) {
        let new_page_num = self.table.pager.get_unused_page_num();
        self.table.pager.get_page(new_page_num).init_leaf();
        self.table.pager.num_pages += 1;

        for i in (0..=page::LEAF_MAX_CELLS).rev() {
            let cell_num = i % page::LEAF_LEFT_SPLIT_COUNT;
            let old_page = self.table.pager.get_page(self.page_num).clone();
            let destination = if i >= page::LEAF_LEFT_SPLIT_COUNT {
                self.table.pager.get_page(new_page_num)
            } else {
                self.table.pager.get_page(self.page_num)
            };

            match i.cmp(&self.cell_num) {
                Ordering::Equal => destination.set_leaf_cell(cell_num, (key, value.serialize())),
                Ordering::Greater => {
                    destination.set_leaf_cell(cell_num, old_page.get_leaf_cell(i - 1))
                }
                Ordering::Less => destination.set_leaf_cell(cell_num, old_page.get_leaf_cell(i)),
            }
        }

        self.table
            .pager
            .get_page(self.page_num)
            .set_leaf_num_cells(page::LEAF_LEFT_SPLIT_COUNT);
        self.table
            .pager
            .get_page(new_page_num)
            .set_leaf_num_cells(page::LEAF_RIGHT_SPLIT_COUNT);

        if self.table.pager.get_page(self.page_num).get_is_root() {
            self.table.new_root(new_page_num);
        } else {
            unimplemented!("Need to implement updating parent after split");
        }
    }
}
