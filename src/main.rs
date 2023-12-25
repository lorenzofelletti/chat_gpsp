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
    utils::str_to_u16_mut_ptr,
};
use psp::sys::{sceGuInit, sceGuTerm, sceKernelDcacheWritebackAll, sceKernelExitGame};
use renderer::Renderer;

use crate::osk::{setup_buffers, setup_gu};

psp::module!("tls-test", 1, 1);

mod net;
mod osk;
mod renderer;

#[allow(dead_code)]
fn select_netconfig() -> i32 {
    unsafe { psp::sys::sceUtilityCheckNetParam(1) }
}

#[allow(dead_code)]
const CHAT_MAX_LENGTH: u16 = 32;
#[allow(dead_code)]
const CHAT_MAX_LENGTH_USIZE: usize = CHAT_MAX_LENGTH as usize;

#[no_mangle]
fn psp_main() {
    psp::enable_home_button();

    unsafe { sceKernelDcacheWritebackAll() }

    unsafe {
        let mut renderer = Renderer::new();

        let mut i = 0;

        psp::dprintln!("cds");
        // loop {
        //     renderer.clear(0xFFFFCA82);
        //     renderer.draw_rect(10, 10 + i, 30, 30, 0xFF00FFFF);

        //     i += 1;

        //     if i == 100 {
        //         break;
        //     }

        //     renderer.swap_buffers_and_wait();
        // }

        let mut out_text: Vec<u16> = Vec::with_capacity(CHAT_MAX_LENGTH_USIZE);
        let out_capacity: i32 = out_text.capacity() as i32;

        let description = str_to_u16_mut_ptr("Ask GPT\0");

        let mut osk_data = default_osk_data(description, out_capacity, out_text.as_mut_ptr());

        let params = &mut default_osk_params(&mut osk_data);

        sceGuInit();
        setup_gu();
        // setup_buffers(
        //     renderer.draw_buffer() as *mut u8,
        //     renderer.disp_buffer() as *mut u8,
        // );

        start_osk(params).expect("failed to start osk");

        let read_text = read_from_osk(params, Some(&renderer)).unwrap_or_default();

        psp::dprintln!("read_text: {:?}", read_text);

        loop {
            psp::dprintln!("read_text: {:?}", read_text);
        }

        let mut i = 0;
        loop {
            renderer.clear(0xFFFFCA82);
            renderer.draw_rect(10, 10 + i, 30, 30, 0xFF00FFFF);

            i = (i + 1) % 100;

            renderer.swap_buffers_and_wait();
        }
    }

    unsafe {
        sceGuTerm();
        sceKernelExitGame();
    };
}

#[allow(dead_code)]
unsafe fn load_modules() {
    psp::sys::sceUtilityLoadNetModule(psp::sys::NetModule::NetCommon);
    psp::sys::sceUtilityLoadNetModule(psp::sys::NetModule::NetInet);
}

#[allow(dead_code)]
unsafe fn init() {
    psp::sys::sceNetInit(0x20000, 0x20, 0x1000, 0x20, 0x1000);
    psp::sys::sceNetInetInit();
    psp::sys::sceNetResolverInit();
    psp::sys::sceNetApctlInit(0x1600, 42);
}
