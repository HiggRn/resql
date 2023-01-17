use std::{cmp::Ordering, mem};

use super::row;

pub enum NodeType {
    Internal,
    Leaf,
}

#[derive(Clone)]
pub struct Page(pub Vec<u8>);

pub const PAGE_SIZE: usize = 4096;

// Common node header layout
const TYPE_OFFSET: usize = 0;
const TYPE_SIZE: usize = mem::size_of::<u8>();
const IS_ROOT_OFFSET: usize = TYPE_SIZE;
const IS_ROOT_SIZE: usize = mem::size_of::<u8>();
const PARENT_OFFSET: usize = IS_ROOT_OFFSET + IS_ROOT_SIZE;
const PARENT_SIZE: usize = mem::size_of::<usize>();
pub const COMMON_HEADER_SIZE: usize = TYPE_SIZE + IS_ROOT_SIZE + PARENT_SIZE;

// Leaf node header layout
const LEAF_NUM_CELLS_OFFSET: usize = COMMON_HEADER_SIZE;
const LEAF_NUM_CELLS_SIZE: usize = mem::size_of::<usize>();
const LEAF_NEXT_LEAF_OFFSET: usize = LEAF_NUM_CELLS_OFFSET + LEAF_NUM_CELLS_SIZE;
const LEAF_NEXT_LEAF_SIZE: usize = mem::size_of::<usize>();
pub const LEAF_HEADER_SIZE: usize = COMMON_HEADER_SIZE + LEAF_NUM_CELLS_SIZE + LEAF_NEXT_LEAF_SIZE;

// Leaf node body layout
const LEAF_KEY_OFFSET: usize = 0;
const LEAF_KEY_SIZE: usize = mem::size_of::<usize>();
const LEAF_VALUE_OFFSET: usize = LEAF_KEY_SIZE;
const LEAF_VALUE_SIZE: usize = row::ROW_SIZE;
pub const LEAF_CELL_SIZE: usize = LEAF_KEY_SIZE + LEAF_VALUE_SIZE;

pub const LEAF_MAX_CELLS: usize = (PAGE_SIZE - LEAF_HEADER_SIZE) / LEAF_CELL_SIZE;
pub const LEAF_RIGHT_SPLIT_COUNT: usize = (LEAF_MAX_CELLS + 1) / 2;
pub const LEAF_LEFT_SPLIT_COUNT: usize = LEAF_MAX_CELLS + 1 - LEAF_RIGHT_SPLIT_COUNT;

// Internal node header layout
const INTERNAL_NUM_KEYS_OFFSET: usize = COMMON_HEADER_SIZE;
const INTERNAL_NUM_KEYS_SIZE: usize = mem::size_of::<usize>();
const INTERNAL_RIGHT_CHILD_OFFSET: usize = INTERNAL_NUM_KEYS_OFFSET + INTERNAL_NUM_KEYS_SIZE;
const INTERNAL_RIGHT_CHILD_SIZE: usize = mem::size_of::<usize>();
const INTERNAL_HEADER_SIZE: usize =
    COMMON_HEADER_SIZE + INTERNAL_NUM_KEYS_SIZE + INTERNAL_RIGHT_CHILD_SIZE;

// Internal node body layout
const INTERNAL_KEY_SIZE: usize = mem::size_of::<usize>();
const INTERNAL_CHILD_SIZE: usize = mem::size_of::<usize>();
const INTERNAL_CELL_SIZE: usize = INTERNAL_KEY_SIZE + INTERNAL_CHILD_SIZE;

pub const INTERNAL_MAX_CELLS: usize = 3; // keep this small for testing

// Common node methods
impl Page {
    pub fn get_type(&self) -> NodeType {
        match self.0[TYPE_OFFSET] {
            0 => NodeType::Internal,
            _ => NodeType::Leaf,
        }
    }

    pub fn get_is_root(&self) -> bool {
        !matches!(self.0[IS_ROOT_OFFSET], 0)
    }

    pub fn get_parent(&self) -> usize {
        let start = PARENT_OFFSET;
        let end = start + PARENT_SIZE;

        usize::from_ne_bytes(self.0[start..end].try_into().unwrap())
    }

    pub fn get_max_key(&self) -> usize {
        match self.get_type() {
            NodeType::Internal => self.get_internal_key(self.get_internal_num_keys() - 1),
            NodeType::Leaf => self.get_leaf_key(self.get_leaf_num_cells() - 1),
        }
    }

    pub fn set_type(&mut self, node_type: NodeType) {
        self.0[TYPE_OFFSET] = match node_type {
            NodeType::Internal => 0,
            NodeType::Leaf => 1,
        };
    }

    pub fn set_is_root(&mut self, is_root: bool) {
        self.0[IS_ROOT_OFFSET] = match is_root {
            false => 0,
            true => 1,
        };
    }

    pub fn set_parent(&mut self, parent_page_num: usize) {
        let start = PARENT_OFFSET;
        let end = start + PARENT_SIZE;
        self.0[start..end].clone_from_slice(&parent_page_num.to_ne_bytes());
    }
}

// Leaf node methods
impl Page {
    pub fn init_leaf(&mut self) {
        self.0 = vec![0; PAGE_SIZE];
        self.set_type(NodeType::Leaf);
        self.set_is_root(false);
        self.set_leaf_num_cells(0);
        self.set_leaf_next_leaf(0); // 0 represents no sibling. page 0 is reserved for root
    }

    pub fn get_leaf_num_cells(&self) -> usize {
        let start = LEAF_NUM_CELLS_OFFSET;
        let end = start + LEAF_NUM_CELLS_SIZE;

        usize::from_ne_bytes(self.0[start..end].try_into().unwrap())
    }

    pub fn get_leaf_next_leaf(&self) -> usize {
        let start = LEAF_NEXT_LEAF_OFFSET;
        let end = start + LEAF_NEXT_LEAF_SIZE;

        usize::from_ne_bytes(self.0[start..end].try_into().unwrap())
    }

    pub fn get_leaf_cell(&self, cell_num: usize) -> (usize, Vec<u8>) {
        (self.get_leaf_key(cell_num), self.get_leaf_value(cell_num))
    }

    pub fn get_leaf_key(&self, cell_num: usize) -> usize {
        let start = LEAF_HEADER_SIZE + cell_num * LEAF_CELL_SIZE + LEAF_KEY_OFFSET;
        let end = start + LEAF_KEY_SIZE;

        usize::from_ne_bytes(self.0[start..end].try_into().unwrap())
    }

    pub fn get_leaf_value(&self, cell_num: usize) -> Vec<u8> {
        let start = LEAF_HEADER_SIZE + cell_num * LEAF_CELL_SIZE + LEAF_VALUE_OFFSET;
        let end = start + LEAF_VALUE_SIZE;

        Vec::from(&self.0[start..end])
    }

    pub fn set_leaf_num_cells(&mut self, num_cells: usize) {
        let start = LEAF_NUM_CELLS_OFFSET;
        let end = start + LEAF_NUM_CELLS_SIZE;
        self.0[start..end].clone_from_slice(&num_cells.to_ne_bytes());
    }

    pub fn set_leaf_next_leaf(&mut self, next_page_num: usize) {
        let start = LEAF_NEXT_LEAF_OFFSET;
        let end = start + LEAF_NEXT_LEAF_SIZE;
        self.0[start..end].clone_from_slice(&next_page_num.to_ne_bytes());
    }

    pub fn set_leaf_cell(&mut self, cell_num: usize, (key, value): (usize, Vec<u8>)) {
        self.set_leaf_key(cell_num, key);
        self.set_leaf_value(cell_num, value);
    }

    pub fn set_leaf_key(&mut self, cell_num: usize, key: usize) {
        let start = LEAF_HEADER_SIZE + cell_num * LEAF_CELL_SIZE;
        let end = start + LEAF_KEY_SIZE;
        self.0[start..end].clone_from_slice(&key.to_ne_bytes());
    }

    pub fn set_leaf_value(&mut self, cell_num: usize, value: Vec<u8>) {
        let start = LEAF_HEADER_SIZE + cell_num * LEAF_CELL_SIZE + LEAF_KEY_SIZE;
        let end = start + LEAF_VALUE_SIZE;
        self.0[start..end].copy_from_slice(&value);
    }

    pub fn leaf_find(&self, key: usize) -> usize {
        let mut min_index = 0;
        let mut one_past_max_index = self.get_leaf_num_cells();
        while one_past_max_index != min_index {
            let index = (min_index + one_past_max_index) / 2;
            match self.get_leaf_key(index).cmp(&key) {
                Ordering::Equal => return index,
                Ordering::Greater => one_past_max_index = index,
                Ordering::Less => min_index = index + 1,
            }
        }

        min_index
    }
}

// Internal node methods
impl Page {
    pub fn init_internal(&mut self) {
        self.0 = vec![0; PAGE_SIZE];
        self.set_type(NodeType::Internal);
        self.set_is_root(false);
        self.set_internal_num_keys(0);
    }

    pub fn get_internal_num_keys(&self) -> usize {
        let start = INTERNAL_NUM_KEYS_OFFSET;
        let end = start + INTERNAL_NUM_KEYS_SIZE;
        usize::from_ne_bytes(self.0[start..end].try_into().unwrap())
    }

    pub fn get_internal_right_child(&self) -> usize {
        let start = INTERNAL_RIGHT_CHILD_OFFSET;
        let end = start + INTERNAL_RIGHT_CHILD_SIZE;
        usize::from_ne_bytes(self.0[start..end].try_into().unwrap())
    }

    pub fn get_internal_key(&self, key_num: usize) -> usize {
        let start = INTERNAL_HEADER_SIZE + key_num * INTERNAL_CELL_SIZE + INTERNAL_CHILD_SIZE;
        let end = start + INTERNAL_KEY_SIZE;
        usize::from_ne_bytes(self.0[start..end].try_into().unwrap())
    }

    pub fn get_internal_child(&self, child_num: usize) -> usize {
        let num_keys = self.get_internal_num_keys();
        match child_num.cmp(&num_keys) {
            Ordering::Greater => {
                crate::error(
                    format!("trying to access child '{child_num}' with only '{num_keys}' keys.")
                        .as_str(),
                );
                std::process::exit(1);
            }
            Ordering::Equal => self.get_internal_right_child(),
            Ordering::Less => {
                let start = INTERNAL_HEADER_SIZE + child_num * INTERNAL_CELL_SIZE;
                let end = start + INTERNAL_CHILD_SIZE;
                usize::from_ne_bytes(self.0[start..end].try_into().unwrap())
            }
        }
    }

    pub fn set_internal_num_keys(&mut self, num_keys: usize) {
        let start = INTERNAL_NUM_KEYS_OFFSET;
        let end = start + INTERNAL_NUM_KEYS_SIZE;
        self.0[start..end].copy_from_slice(num_keys.to_ne_bytes().as_slice());
    }

    pub fn set_internal_right_child(&mut self, right_child_page_num: usize) {
        let start = INTERNAL_RIGHT_CHILD_OFFSET;
        let end = start + INTERNAL_RIGHT_CHILD_SIZE;
        self.0[start..end].copy_from_slice(right_child_page_num.to_ne_bytes().as_slice());
    }

    pub fn set_internal_key(&mut self, key_num: usize, key: usize) {
        let start = INTERNAL_HEADER_SIZE + key_num * INTERNAL_CELL_SIZE + INTERNAL_CHILD_SIZE;
        let end = start + INTERNAL_KEY_SIZE;
        self.0[start..end].copy_from_slice(key.to_ne_bytes().as_slice());
    }

    pub fn set_internal_child(&mut self, child_num: usize, child: usize) {
        let num_keys = self.get_internal_num_keys();
        match child_num.cmp(&num_keys) {
            Ordering::Greater => {
                crate::error(
                    format!("trying to access child '{child_num}' with only '{num_keys}' keys.")
                        .as_str(),
                );
                std::process::exit(1);
            }
            Ordering::Equal => self.set_internal_right_child(child),
            Ordering::Less => {
                let start = INTERNAL_HEADER_SIZE + child_num * INTERNAL_CELL_SIZE;
                let end = start + INTERNAL_CHILD_SIZE;
                self.0[start..end].copy_from_slice(child.to_ne_bytes().as_slice());
            }
        }
    }

    pub fn internal_find(&self, key: usize) -> usize {
        let mut min_index = 0;
        let mut max_index = self.get_internal_num_keys(); // there is one more child than key
        while max_index != min_index {
            let index = (min_index + max_index) / 2;
            match self.get_internal_key(index).cmp(&key) {
                Ordering::Equal | Ordering::Greater => max_index = index,
                Ordering::Less => min_index = index + 1,
            }
        }

        min_index
    }

    pub fn internal_update_key(&mut self, old_key: usize, new_key: usize) {
        let old_child_index = self.internal_find(old_key);
        self.set_internal_key(old_child_index, new_key);
    }
}
