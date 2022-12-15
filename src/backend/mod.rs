pub mod table;

pub type Row = row::Row;
pub type Table = table::Table;

mod cursor;
mod row;
mod pager;

type Cursor<'a> = cursor::Cursor<'a>;
type Pager = pager::Pager;

pub struct Page(Vec<u8>);
impl Page {
    const PAGE_SIZE: usize = 4096;
    const MAX_ROWS: usize = Page::PAGE_SIZE / Row::ROW_SIZE;
}
