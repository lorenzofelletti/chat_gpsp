#![no_std]
#![no_main]
#![feature(c_void_variant)]

extern crate alloc;

static mut VRAM: *mut u32 = 0x4000_0000 as *mut u32;
static mut LIST: psp::Align16<[u32; 262_144]> = psp::Align16([0; 262_144]);

use core::ffi::c_void;

use alloc::vec::Vec;
use osk::{
    main_fn,
    prelude::{default_osk_data, default_osk_params},
    read_from_osk, start_osk,
    utils::str_to_u16_mut_ptr,
};
use psp::{
    sys::{
        self, sceDisplayWaitVblank, sceDisplayWaitVblankStart, sceGuDisable, sceGuDisplay,
        sceGuFinish, sceGuInit, sceGuStart, sceGuSwapBuffers, sceGuSync, sceGuTerm,
        sceKernelDcacheWritebackAll, sceKernelDcacheWritebackInvalidateAll, sceKernelDelayThread,
        sceKernelExitGame, sceKernelIcacheInvalidateAll,
    },
    BUF_WIDTH, SCREEN_HEIGHT, SCREEN_WIDTH,
};
use renderer::Renderer;

use crate::osk::{setup_gu, x};

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

        start_osk(params).expect("failed to start osk");

        let read_text = read_from_osk(params, Some(&renderer)).unwrap_or_default();

        sceGuFinish();
        sceDisplayWaitVblankStart();
        sceGuTerm();
        sceDisplayWaitVblankStart();

        // sceGuDisable(sys::GuState::ScissorTest);
        // sceGuFinish();
        // sceGuSwapBuffers();
        // sceGuFinish();
        // sceGuTerm();
        renderer = Renderer::new();
        renderer.clear(0xFFFFCA82);

        //sceGuSync(sys::GuSyncMode::Finish, sys::GuSyncBehavior::Wait);
        ////sceDisplayWaitVblank();
        //x();
        //sys::sceGuDisplay(false);

        //sceGuTerm();
        //sceKernelDcacheWritebackInvalidateAll();
        //sceKernelIcacheInvalidateAll();

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

    main_fn();

    unsafe {
        sys::sceDisplaySetMode(
            sys::DisplayMode::Lcd,
            SCREEN_WIDTH as usize,
            SCREEN_HEIGHT as usize,
        );
        //panic!("");

        // Cache-through address
        VRAM = (0x4000_0000u32 | sys::sceGeEdramGetAddr() as u32) as *mut u32;

        sys::sceDisplaySetFrameBuf(
            VRAM as *const u8,
            BUF_WIDTH as usize,
            sys::DisplayPixelFormat::Psm8888,
            sys::DisplaySetBufSync::NextFrame,
        );

        //panic!("");

        let mut i = 0;
        loop {
            sys::sceDisplayWaitVblankStart();
            //panic!("");
            for pos in 0..255 {
                let color = wheel(pos);

                //panic!("");

                for i in 0..(BUF_WIDTH * SCREEN_HEIGHT) {
                    *VRAM.add(i as usize) = color;
                }
            }
            i += 1;
            if i == 5 {
                break;
            }
        }
    }

    unsafe {
        sceGuTerm();
        sceKernelExitGame();
    };

    //let string = "Hello, world!";
    //let str_as_u16 = str_to_u16_mut_ptr(string);

    // let vec: Vec<char> = Vec::with_capacity(string.len());
    // let mut str_as_vec = Vec::new();

    // for i in 0..string.len() {
    // let char = unsafe { *str_as_u16.add(i) as u8 as char };
    // str_as_vec.push(char);
    // }
    // let res_string: String = String::from_iter(str_as_vec);
    // psp::dprintln!("{}", res_string);

    //assert_eq!(string.to_owned(), res_string);

    // let reconstructed_string = mut_ptr_u16_to_vec_u16(str_as_u16, string.len());
    // psp::dprint!("{:?}", reconstructed_string);
    // for c in reconstructed_string.iter() {
    // let char = *c as u8 as char;
    // psp::dprint!("{}", char);
    // }

    // psp::dprintln!("");

    //let reconstructed_string = String::from_iter(mut_ptr_u16_to_vec_char(str_as_u16, string.len()));
    //assert_eq!(string.to_owned(), reconstructed_string);

    //x();
    //unsafe { sceKernelDelayThread(100_000) };
    //unsafe { psp::sys::sceGuInit() };
    //x();
    //main_fn();

    //x();
    //psp::dprint!("Hello, world!");
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

fn wheel(mut pos: u8) -> u32 {
    pos = 255 - pos;
    if pos < 85 {
        u32::from_be_bytes([255 - pos * 3, 0, pos * 3, 255])
    } else if pos < 170 {
        pos -= 85;
        u32::from_be_bytes([0, pos * 3, 255 - pos * 3, 255])
    } else {
        pos -= 170;
        u32::from_be_bytes([pos * 3, 255 - pos * 3, 0, 255])
    }
}
