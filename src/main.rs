#![no_std]
#![no_main]
#![feature(c_void_variant)]

extern crate alloc;

#[allow(dead_code)]
static mut VRAM: *mut u32 = 0x4000_0000 as *mut u32;
#[allow(dead_code)]
static mut LIST: psp::Align16<[u32; 262_144]> = psp::Align16([0; 262_144]);

use alloc::vec::Vec;
use osk::{
    prelude::{default_osk_data, default_osk_params},
    read_from_osk, start_osk,
};
use psp::sys::{sceGuTerm, sceKernelDcacheWritebackAll, sceKernelExitGame};

use crate::{osk::setup_gu, utils::str_to_u16_mut_ptr};

psp::module!("tls-test", 1, 1);

mod net;
mod osk;
pub mod utils;

#[allow(dead_code)]
fn select_netconfig() -> i32 {
    unsafe { psp::sys::sceUtilityCheckNetParam(1) }
}

#[allow(dead_code)]
const CHAT_MAX_LENGTH: u16 = 128;
#[allow(dead_code)]
const CHAT_MAX_LENGTH_USIZE: usize = CHAT_MAX_LENGTH as usize;

#[no_mangle]
fn psp_main() {
    psp::enable_home_button();

    unsafe {
        sceKernelDcacheWritebackAll();

        let mut out_text: Vec<u16> = Vec::with_capacity(CHAT_MAX_LENGTH_USIZE);
        let out_capacity: i32 = out_text.capacity() as i32;

        let description = str_to_u16_mut_ptr("Ask GPT\0");
        let mut osk_data = default_osk_data(description, out_capacity, out_text.as_mut_ptr());

        let params = &mut default_osk_params(&mut osk_data);

        setup_gu();

        start_osk(params).expect("failed to start osk");

        let read_text = read_from_osk(params).unwrap_or_default();

        psp::dprintln!("read_text: {:?}", read_text);
    }

    unsafe {
        sceGuTerm();
        sceKernelExitGame();
    };
}
