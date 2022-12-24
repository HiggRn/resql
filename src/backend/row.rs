use std::cmp;
use std::mem;

use byteorder::{ByteOrder, LittleEndian};

pub struct Row {
    pub id: u32,
    pub username: String,
    pub email: String,
}

pub const MAX_USERNAME: usize = 31;
pub const MAX_EMAIL: usize = 255;
const ID_SIZE: usize = mem::size_of::<u32>();
const USERNAME_SIZE: usize = MAX_USERNAME + 1;
const EMAIL_SIZE: usize = MAX_EMAIL + 1;
const USERNAME_OFFSET: usize = ID_SIZE;
const EMAIL_OFFSET: usize = USERNAME_OFFSET + USERNAME_SIZE;
pub const ROW_SIZE: usize = ID_SIZE + USERNAME_SIZE + EMAIL_SIZE;

impl Row {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = vec![0; ROW_SIZE];
        LittleEndian::write_u32(&mut buf[0..ID_SIZE], self.id);
        Self::write_string(&mut buf, USERNAME_OFFSET, &self.username, USERNAME_SIZE);
        Self::write_string(&mut buf, EMAIL_OFFSET, &self.email, EMAIL_SIZE);

        buf
    }

    pub fn deserialize(buf: &[u8], pos: usize) -> Self {
        let mut bytes = vec![0; ROW_SIZE];
        bytes.clone_from_slice(&buf[pos..pos + ROW_SIZE]);

        let id = LittleEndian::read_u32(&bytes[0..ID_SIZE]);
        let username = Self::read_string(&bytes, USERNAME_OFFSET, USERNAME_SIZE);
        let email = Self::read_string(&bytes, EMAIL_OFFSET, EMAIL_SIZE);

        Self {
            id,
            username,
            email,
        }
    }

    fn write_string(buf: &mut [u8], pos: usize, s: &str, length: usize) {
        let bytes = s.as_bytes();
        let len = bytes.len();
        buf[pos..pos + len].copy_from_slice(bytes);
        buf[pos + len..pos + length].copy_from_slice(&vec![0; length - len]);
    }

    fn read_string(buf: &Vec<u8>, pos: usize, length: usize) -> String {
        // buf.len() MUST be greater than pos
        let len = cmp::min(length, buf.len() - pos);
        let mut bytes = vec![0; len];
        bytes.clone_from_slice(&buf[pos..pos + len]);

        String::from_utf8(bytes).unwrap()
    }
}
