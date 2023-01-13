use std::cmp;
use std::mem;

#[derive(Clone, Default)]
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
        buf[0..ID_SIZE].clone_from_slice(&self.id.to_ne_bytes());
        Self::write_string(&mut buf, USERNAME_OFFSET, &self.username, USERNAME_SIZE);
        Self::write_string(&mut buf, EMAIL_OFFSET, &self.email, EMAIL_SIZE);

        buf
    }

    pub fn deserialize(buf: &[u8]) -> Self {
        let mut bytes = vec![0; ROW_SIZE];
        bytes.clone_from_slice(&buf[0..ROW_SIZE]);

        let id = u32::from_ne_bytes(bytes[0..ID_SIZE].try_into().unwrap());
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
