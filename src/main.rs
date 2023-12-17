#![no_std]
#![no_main]
#![feature(c_void_variant)]

extern crate alloc;

use osk::main_fn;
use psp::sys::{sceKernelDcacheWritebackAll, sceKernelDelayThread};

use crate::osk::x;

psp::module!("tls-test", 1, 1);

mod net;
mod osk;

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
    main_fn();

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
