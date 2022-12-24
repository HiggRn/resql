pub mod row;
pub mod table;

pub type Row = row::Row;
pub type Table = table::Table;

mod cursor;
mod pager;

type Cursor<'a> = cursor::Cursor<'a>;
type Pager = pager::Pager;

type Page = Vec<u8>;
mod page {
    use super::row;

    pub const PAGE_SIZE: usize = 4096;
    pub const MAX_ROWS: usize = PAGE_SIZE / row::ROW_SIZE;
}
