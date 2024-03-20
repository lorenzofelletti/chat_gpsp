use alloc::vec::Vec;

pub mod dns;

pub trait SocketBuffer {
    fn new() -> Self
    where
        Self: Sized;
    fn append_buffer(&mut self, buf: &[u8]);
    fn shift_left_buffer(&mut self, amount: usize);
    fn clear(&mut self) {
        self.shift_left_buffer(self.len());
    }
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn as_slice(&self) -> &[u8];
    fn len(&self) -> usize;
}

impl SocketBuffer for Vec<u8> {
    #[inline]
    fn new() -> Self {
        Vec::new()
    }

    #[inline]
    fn append_buffer(&mut self, buf: &[u8]) {
        self.append(&mut buf.to_vec());
    }

    fn shift_left_buffer(&mut self, amount: usize) {
        // shift the buffer to the left by amount
        if self.len() <= amount {
            self.clear();
        } else {
            self.drain(..amount);
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn clear(&mut self) {
        self.clear();
    }

    #[inline]
    fn as_slice(&self) -> &[u8] {
        self.as_slice()
    }
}
