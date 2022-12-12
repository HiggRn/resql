use std::cmp;
use std::mem;
use std::ops::{Index, IndexMut, Range};

use byteorder::{LittleEndian, ByteOrder};

pub struct Row {
    pub id: u32,
    pub username: String,
    pub email: String
}

impl Row {
    pub const MAX_USERNAME: usize = 31;
    pub const MAX_EMAIL: usize = 255;
    const ID_SIZE: usize = mem::size_of::<u32>();
    const USERNAME_SIZE: usize = Self::MAX_USERNAME + 1;
    const EMAIL_SIZE: usize = Self::MAX_EMAIL + 1;
    const USERNAME_OFFSET: usize = Self::ID_SIZE;
    const EMAIL_OFFSET: usize = Self::USERNAME_OFFSET + Self::USERNAME_SIZE;
    pub const ROW_SIZE: usize = Self::ID_SIZE + Self::USERNAME_SIZE + Self::EMAIL_SIZE;

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = vec![0; Self::ROW_SIZE];
        LittleEndian::write_u32(
            &mut buf.index_mut(Range {
                start: 0,
                end: Self::ID_SIZE,
            }),
            self.id,
        );
        Self::write_string(&mut buf, Self::USERNAME_OFFSET, &self.username, Self::USERNAME_SIZE);
        Self::write_string(&mut buf, Self::EMAIL_OFFSET, &self.email, Self::EMAIL_SIZE);
        return buf;
    }

    pub fn deserialize(buf: &Vec<u8>, pos: usize) -> Self {
        let mut bytes = vec![0; Self::ROW_SIZE];
        bytes.clone_from_slice(buf.index(Range {
            start: pos,
            end: pos + Self::ROW_SIZE,
        }));

        let id = LittleEndian::read_u32(&bytes[0..4]);
        let username = Self::read_string(&bytes, Self::USERNAME_OFFSET, Self::USERNAME_SIZE);
        let email = Self::read_string(&bytes, Self::EMAIL_OFFSET, Self::EMAIL_SIZE);
        Self {
            id,
            username,
            email,
        }
    }

    fn write_string(buf: &mut Vec<u8>, pos: usize, s: &str, length: usize) {
        let bytes = s.as_bytes();
        let len = bytes.len();
        buf[pos..pos + len].copy_from_slice(bytes);
        buf[pos + len..pos + length].copy_from_slice(&vec![0u8; length - len]);
    }

    fn read_string(buf: &Vec<u8>, pos: usize, length: usize) -> String { // buf.len() MUST be greater than pos
        let len = cmp::min(length, buf.len() - pos);
        let mut bytes = vec![0; len];
        bytes.clone_from_slice(buf.index(Range { start: pos, end: pos + len}));
        return String::from_utf8(bytes).unwrap();
    }
}
