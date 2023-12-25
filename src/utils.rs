use alloc::{vec, vec::Vec};
use psp::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub const BUF_WIDTH: u32 = 512;

pub const BUF_WIDTH_I32: i32 = BUF_WIDTH as i32;
pub const BUF_WIDTH_USIZE: usize = BUF_WIDTH as usize;

pub const SCREEN_WIDTH_I32: i32 = SCREEN_WIDTH as i32;
pub const SCREEN_WIDTH_USIZE: usize = SCREEN_WIDTH as usize;

pub const SCREEN_HEIGHT_I32: i32 = SCREEN_HEIGHT as i32;
pub const SCREEN_HEIGHT_USIZE: usize = SCREEN_HEIGHT as usize;

/// Convert a mutable pointer to a u16 to a Vec<u16>.
#[allow(unused)]
#[inline(always)]
pub fn mut_ptr_u16_to_vec_u16(ptr: *mut u16, len: usize) -> Vec<u16> {
    #[allow(unused)]
    let vec: Vec<u16> = Vec::with_capacity(len); // otherwise it doesn't work
    let mut vec = Vec::new();
    for i in 0..len {
        let char = unsafe { *ptr.add(i) };
        vec.push(char);
    }
    vec
}

#[inline(always)]
pub fn str_to_u16_mut_ptr(s: &str) -> *mut u16 {
    let mut vec = vec![0u16; s.len()];
    for (i, c) in s.char_indices() {
        vec[i] = c as u16;
    }
    vec.as_mut_ptr()
}

#[inline(always)]
pub fn mut_ptr_u16_to_vec_char(ptr: *mut u16, len: usize) -> Vec<char> {
    #[allow(unused)]
    let vec: Vec<char> = Vec::with_capacity(len);
    let mut vec = Vec::new();

    for i in 0..len {
        let char = unsafe { *ptr.add(i) as u8 as char };
        vec.push(char);
        if char == '\0' {
            break;
        }
    }
    vec
}
