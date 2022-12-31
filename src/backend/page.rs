use std::mem;

use byteorder::{ByteOrder, LittleEndian};

use super::row;

pub enum NodeType {
    Internal,
    Leaf,
}

pub struct Page(pub Vec<u8>);

// Common node header layout
const TYPE_OFFSET: usize = 0;
const TYPE_SIZE: usize = mem::size_of::<u8>();
const IS_ROOT_OFFSET: usize = TYPE_SIZE;
const IS_ROOT_SIZE: usize = mem::size_of::<u8>();
const PARENT_PTR_OFFSET: usize = IS_ROOT_OFFSET + IS_ROOT_SIZE;
const PARENT_PTR_SIZE: usize = mem::size_of::<&Page>();
pub const COMMON_HEADER_SIZE: usize = TYPE_SIZE + IS_ROOT_SIZE + PARENT_PTR_SIZE;

// Leaf node header layout
const LEAF_NUM_CELLS_OFFSET: usize = COMMON_HEADER_SIZE;
const LEAF_NUM_CELLS_SIZE: usize = mem::size_of::<u32>();
pub const LEAF_HEADER_SIZE: usize = COMMON_HEADER_SIZE + LEAF_NUM_CELLS_SIZE;

// Leaf node body layout
const LEAF_KEY_OFFSET: usize = 0;
const LEAF_KEY_SIZE: usize = mem::size_of::<u32>();
const LEAF_VALUE_OFFSET: usize = LEAF_KEY_SIZE;
const LEAF_VALUE_SIZE: usize = row::ROW_SIZE;
pub const LEAF_CELL_SIZE: usize = LEAF_KEY_SIZE + LEAF_VALUE_SIZE;
pub const LEAF_MAX_CELLS: usize = (PAGE_SIZE - LEAF_HEADER_SIZE) / LEAF_CELL_SIZE;

pub const PAGE_SIZE: usize = 4096;

impl Page {
    pub fn init_leaf_node(&mut self) {
        self.set_type(NodeType::Leaf);
        self.set_leaf_node_num_cells(0);
    }

    pub fn get_type(&self) -> NodeType {
        match self.0[TYPE_OFFSET] {
            0 => NodeType::Internal,
            _ => NodeType::Leaf,
        }
    }

    pub fn get_leaf_node_num_cells(&self) -> u32 {
        let start = LEAF_NUM_CELLS_OFFSET;
        let end = LEAF_NUM_CELLS_OFFSET + LEAF_NUM_CELLS_SIZE;

        LittleEndian::read_u32(&self.0[start..end])
    }

    pub fn get_leaf_node_cell(&self, cell_num: u32) -> (u32, Vec<u8>) {
        (
            self.get_leaf_node_key(cell_num),
            self.get_leaf_node_value(cell_num),
        )
    }

    pub fn get_leaf_node_key(&self, cell_num: u32) -> u32 {
        let start = LEAF_HEADER_SIZE + (cell_num as usize * LEAF_CELL_SIZE);
        let end = start + LEAF_KEY_SIZE;

        LittleEndian::read_u32(&self.0[start..end])
    }

    pub fn get_leaf_node_value(&self, cell_num: u32) -> Vec<u8> {
        let start = LEAF_HEADER_SIZE + (cell_num as usize * LEAF_CELL_SIZE) + LEAF_KEY_SIZE;
        let end = start + LEAF_VALUE_SIZE;

        Vec::from(&self.0[start..end])
    }

    pub fn set_type(&mut self, node_type: NodeType) {
        self.0[TYPE_OFFSET] = match node_type {
            NodeType::Internal => 0,
            NodeType::Leaf => 1,
        };
    }

    pub fn set_leaf_node_num_cells(&mut self, num_cells: u32) {
        let start = LEAF_NUM_CELLS_OFFSET;
        let end = LEAF_NUM_CELLS_OFFSET + LEAF_NUM_CELLS_SIZE;
        LittleEndian::write_u32(&mut self.0[start..end], num_cells);
    }

    pub fn set_leaf_node_cell(&mut self, cell_num: u32, (key, value): (u32, Vec<u8>)) {
        self.set_leaf_node_key(cell_num, key);
        self.set_leaf_node_value(cell_num, value);
    }

    pub fn set_leaf_node_key(&mut self, cell_num: u32, key: u32) {
        let start = LEAF_HEADER_SIZE + (cell_num as usize * LEAF_CELL_SIZE);
        let end = start + LEAF_KEY_SIZE;
        LittleEndian::write_u32(&mut self.0[start..end], key);
    }

    pub fn set_leaf_node_value(&mut self, cell_num: u32, value: Vec<u8>) {
        let start = LEAF_HEADER_SIZE + (cell_num as usize * LEAF_CELL_SIZE) + LEAF_KEY_SIZE;
        let end = start + LEAF_VALUE_SIZE;
        self.0[start..end].copy_from_slice(&value);
    }

    pub fn find(&self, key: u32) -> u32 {
        let mut min_index = 0;
        let mut one_past_max_index = self.get_leaf_node_num_cells();
        while one_past_max_index != min_index {
            let index = (min_index + one_past_max_index) / 2;
            let key_at_index = self.get_leaf_node_key(index);
            if key == key_at_index {
                return index;
            } else if key < key_at_index {
                one_past_max_index = index;
            } else {
                min_index = index + 1;
            }
        }

        min_index
    }

    pub fn print_leaf_node(&mut self) {
        println!("Tree:\n");
        let num_cells = self.get_leaf_node_num_cells();
        println!("leaf (size {num_cells})");
        for i in 0..num_cells {
            let key = self.get_leaf_node_key(i);
            println!("  - {i} : {key}");
        }
    }
}
