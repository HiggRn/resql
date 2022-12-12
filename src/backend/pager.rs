use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

use super::{Page, Row};

pub struct Pager {
    file_descriptor: File,
    file_length: usize,
    pages: Vec<Option<Page>>
}

impl Pager {    
    pub const MAX_PAGES: usize = 100;

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

        let mut pages = Vec::with_capacity(Self::MAX_PAGES);
        for _ in 0..Self::MAX_PAGES {
            pages.push(None);
        }

        Self {
            file_descriptor,
            file_length,
            pages
        }
    }

    pub fn get_num_rows(&self) -> usize {
        self.file_length / Row::ROW_SIZE
    }

    pub fn get_page(&mut self, page_num: usize) -> &mut Page {
        if page_num > Self::MAX_PAGES {
            crate::error(format!("page number '{page_num}' is out of bound").as_str());
            std::process::exit(1);
        }

        if let None = self.pages[page_num] { // cache miss
            self.pages[page_num] = Some(Page{ 0: vec![0; Page::PAGE_SIZE] }); // initialize empty page
            let mut num_pages = self.file_length / Page::PAGE_SIZE;

            if self.file_length % Page::PAGE_SIZE != 0 { // partial page
                num_pages += 1;
            }

            if page_num <= num_pages {
                self.file_descriptor
                    .seek(SeekFrom::Start(
                        (page_num * Page::PAGE_SIZE) as u64
                    ))
                    .unwrap_or_else(|e| {
                        crate::error(format!("failed to seek file. {e}").as_str());
                        std::process::exit(1);
                    });
                
                let mut buf = [0u8; Page::PAGE_SIZE];
                self.file_descriptor
                    .read(&mut buf)
                    .unwrap_or_else(|e| {
                        crate::error(format!("failed to read file. {e}").as_str());
                        std::process::exit(1);
                    });
                self.pages[page_num] = Some(Page(Vec::from(buf)));
            }
        }

        self.pages[page_num].as_mut().unwrap()
    }

    pub fn flush(&mut self, page_num: usize, size: usize) {
        if let None = self.pages[page_num] {
            return;
        }

        self.file_descriptor
            .seek(SeekFrom::Start(
                (page_num * Page::PAGE_SIZE) as u64
            ))
            .unwrap_or_else(|e| {
                crate::error(format!("failed to seek file. {e}").as_str());
                std::process::exit(1);
            });
        
        self.file_descriptor
            .write(
            self.pages[page_num]
                    .as_ref()
                    .unwrap()
                    .0[0..size]
                    .as_ref()
            )
            .unwrap_or_else(|e| {
                crate::error(format!("failed to write to file. {e}").as_str());
                std::process::exit(1);
            });

        self.pages[page_num] = None;
    }
}
