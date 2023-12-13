#![no_std]
#![no_main]
#![feature(c_void_variant)]

extern crate alloc;

use osk::main_fn;

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

    main_fn();
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
