use std::io;

#[derive(Default)]
pub struct InputBuffer {
    buffer: String,
    input_length: usize,
}

impl InputBuffer {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            input_length: 0,
        }
    }

    pub fn read(&mut self) {
        io::stdin()
            .read_line(&mut self.buffer)
            .expect("[ERROR]failed to read from stdin");
        if !self.buffer.is_empty() {
            self.input_length = self.buffer.len();
        }
    }

    pub fn get_input(&self) -> Option<&str> {
        if self.buffer.is_empty() {
            None
        } else {
            Some(self.buffer.trim_end())
        }
    }
}
