use alloc::vec::Vec;

pub mod dns;

/// A trait for a buffer that can be used with a socket
pub trait SocketBuffer {
    fn new() -> Self
    where
        Self: Sized;

    /// Append a buffer to the end of itself
    fn append_buffer(&mut self, buf: &[u8]);

    /// Shift the buffer to the left by amount.
    /// This is used to remove data from the buffer.
    fn shift_left_buffer(&mut self, amount: usize);
    /// Clear the buffer
    fn clear(&mut self) {
        self.shift_left_buffer(self.len());
    }
    /// Check if the buffer is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Get the buffer as a slice
    fn as_slice(&self) -> &[u8];
    /// Get the length of the buffer
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
