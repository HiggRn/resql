use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

use super::page::NodeType;
use super::{page, Page};

pub struct Pager {
    file_descriptor: File,
    file_length: usize,
    pub num_pages: usize,
    pages: Vec<Option<Page>>,
}

pub const MAX_PAGES: usize = 100;

impl Pager {
    pub fn new(filename: &str) -> Self {
        let file_descriptor = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filename)
            .unwrap_or_else(move |_| {
                crate::error(format!("can't open '{filename}'").as_str());
                std::process::exit(1);
            });

        let file_length = file_descriptor
            .metadata()
            .unwrap_or_else(move |_| {
                crate::error(format!("can't fetch the metadata of '{filename}'").as_str());
                std::process::exit(1);
            })
            .len() as usize;

        let num_pages = file_length / page::PAGE_SIZE;
        if file_length % page::PAGE_SIZE != 0 {
            crate::error(format!("corrupted file '{filename}'").as_str());
            std::process::exit(1);
        }

        let mut pages = Vec::with_capacity(MAX_PAGES);
        for _ in 0..MAX_PAGES {
            pages.push(None);
        }

        Self {
            file_descriptor,
            file_length,
            num_pages,
            pages,
        }
    }

    pub fn get_unused_page_num(&self) -> usize {
        // Until we start recycling free pages, new pages will always
        // go onto the end of the database file
        self.num_pages
    }

    pub fn copy_page(&self, page_num: usize) -> Option<Page> {
        if page_num > MAX_PAGES {
            crate::error(format!("page number '{page_num}' is out of bound").as_str());
            std::process::exit(1);
        }

        self.pages[page_num].clone()
    }

    pub fn get_page(&mut self, page_num: usize) -> &mut Page {
        if page_num > MAX_PAGES {
            crate::error(format!("page number '{page_num}' is out of bound").as_str());
            std::process::exit(1);
        }

        if self.pages[page_num].is_none() {
            // cache miss
            self.pages[page_num] = Some(Page {
                0: vec![0; page::PAGE_SIZE],
            }); // initialize empty page
            let mut num_pages = self.file_length / page::PAGE_SIZE;

            if self.file_length % page::PAGE_SIZE != 0 {
                // partial page
                num_pages += 1;
            }

            if page_num <= num_pages {
                self.file_descriptor
                    .seek(SeekFrom::Start((page_num * page::PAGE_SIZE) as u64))
                    .unwrap_or_else(|e| {
                        crate::error(format!("failed to seek file. {e}").as_str());
                        std::process::exit(1);
                    });

                let mut buf = [0u8; page::PAGE_SIZE];
                let read_amount = self.file_descriptor.read(&mut buf).unwrap_or_else(|e| {
                    crate::error(format!("failed to read file. {e}").as_str());
                    std::process::exit(1);
                });
                if read_amount > 0 && read_amount < page::PAGE_SIZE {
                    crate::error("partial database file.")
                }

                self.pages[page_num] = Some(Page { 0: Vec::from(buf) });

                if page_num >= self.num_pages {
                    self.num_pages = page_num + 1;
                }
            }
        }

        let page = self.pages[page_num].as_mut().unwrap();

        page
    }

    pub fn flush(&mut self, page_num: usize) {
        if self.pages[page_num].is_none() {
            return;
        }

        self.file_descriptor
            .seek(SeekFrom::Start((page_num * page::PAGE_SIZE) as u64))
            .unwrap_or_else(|e| {
                crate::error(format!("failed to seek file. {e}").as_str());
                std::process::exit(1);
            });

        self.file_descriptor
            .write_all(&self.pages[page_num].as_ref().unwrap().0[0..page::PAGE_SIZE])
            .unwrap_or_else(|e| {
                crate::error(format!("failed to write to file. {e}").as_str());
                std::process::exit(1);
            });

        self.pages[page_num] = None;
    }

    pub fn print(&mut self, page_num: usize, indentation_level: usize) {
        fn indent(level: usize) {
            for _ in 0..level {
                print!("  ");
            }
        }

        let page = self.get_page(page_num).clone();

        match page.get_type() {
            NodeType::Leaf => {
                let num_cells = page.get_leaf_num_cells();
                indent(indentation_level);
                println!("- leaf (size {num_cells})");
                for i in 0..num_cells {
                    indent(indentation_level + 1);
                    println!("- key {}", page.get_leaf_key(i));
                }
            }
            NodeType::Internal => {
                let num_keys = page.get_internal_num_keys();
                indent(indentation_level);
                println!("- internal (size {num_keys})");
                for i in 0..num_keys {
                    let child = page.get_internal_child(i);
                    self.print(child, indentation_level + 1);

                    indent(indentation_level + 1);
                    println!("- key {}", page.get_internal_key(i));
                }
                let right_child = page.get_internal_right_child();
                self.print(right_child, indentation_level + 1);
            }
        }
    }
}
