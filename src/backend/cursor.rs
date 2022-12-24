use super::{page, Row, Table};

pub struct Cursor<'a> {
    table: &'a mut Table,
    page_num: usize,
    cell_num: u32,
    pub end_of_table: bool,
}

impl<'a> Cursor<'a> {
    pub fn from_start(table: &'a mut Table) -> Self {
        let num_cells = table
            .pager
            .get_page(table.root_page_num)
            .get_leaf_node_num_cells();

        Self {
            page_num: table.root_page_num,
            cell_num: 0,
            end_of_table: num_cells == 0,
            table,
        }
    }

    pub fn from_end(table: &'a mut Table) -> Self {
        let cell_num = table
            .pager
            .get_page(table.root_page_num)
            .get_leaf_node_num_cells();

        Self {
            page_num: table.root_page_num,
            cell_num,
            end_of_table: true,
            table,
        }
    }

    pub fn get_row(&mut self) -> Row {
        let page_num = self.page_num;
        let page = self.table.pager.get_page(page_num);

        Row::deserialize(&page.get_leaf_node_value(self.cell_num))
    }

    pub fn advance(&mut self) {
        let page_num = self.page_num;
        let page = self.table.pager.get_page(page_num);
        self.cell_num += 1;
        if self.cell_num >= page.get_leaf_node_num_cells() {
            self.end_of_table = true;
        }
    }

    pub fn leaf_node_insert(&mut self, key: u32, value: &Row) {
        let page = self.table.pager.get_page(self.page_num);
        let num_cells = page.get_leaf_node_num_cells();
        if num_cells as usize >= page::LEAF_MAX_CELLS {
            // Node full
            crate::error("Need to implement splitting a leaf node");
            std::process::exit(1);
        }

        if self.cell_num < num_cells {
            // Make room for new cell
            for i in (self.cell_num..num_cells).rev() {
                let cell = page.get_leaf_node_cell(i);
                page.set_leaf_node_cell(i + 1, cell);
            }
        }

        page.set_leaf_node_num_cells(num_cells + 1);
        page.set_leaf_node_key(self.cell_num, key);
        page.set_leaf_node_value(self.cell_num, value.serialize());
    }
}
