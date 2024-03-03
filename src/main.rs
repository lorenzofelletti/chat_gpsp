#![no_std]
#![no_main]
#![feature(c_void_variant)]

extern crate alloc;

use alloc::vec::Vec;
use net::dns::DnsResolver;
use openai::{OpenAi, OpenAiContext};
use osk::{
    prelude::{default_osk_data, default_osk_params},
    read_from_osk, start_osk,
};
use psp::sys::{sceGuTerm, sceKernelDcacheWritebackAll, sceKernelExitGame};

use crate::{osk::setup_gu, utils::str_to_u16_mut_ptr};

psp::module!("chat-gpsp", 1, 1);

mod net;
mod openai;
mod osk;
pub mod utils;

#[allow(dead_code)]
static mut VRAM: *mut u32 = 0x4000_0000 as *mut u32;
#[allow(dead_code)]
static mut LIST: psp::Align16<[u32; 262_144]> = psp::Align16([0; 262_144]);

#[allow(dead_code)]
const CHAT_MAX_LENGTH: u16 = 128;
#[allow(dead_code)]
const CHAT_MAX_LENGTH_USIZE: usize = CHAT_MAX_LENGTH as usize;

const OPENAI_API_KEY: &'static str = "vuvuzela"; // core::env!("OPENAI_API_KEY");

#[no_mangle]
fn psp_main() {
    psp::enable_home_button();

    unsafe {
        net::utils::load_net_modules();
        psp::dprintln!("Initializing network...");
        net::utils::net_init();

        psp::sys::sceNetApctlConnect(1);
        loop {
            let mut state: psp::sys::ApctlState = core::mem::zeroed();
            psp::sys::sceNetApctlGetState(&mut state);
            if let psp::sys::ApctlState::GotIp = state {
                break;
            }
            psp::sys::sceKernelDelayThread(50_000);
        }
    }

    psp::dprintln!("Connected to network!");

    let mut resolver = DnsResolver::default().expect("failed to create resolver");

    psp::dprintln!("Created resolver!");

    let mut record_read_buf = OpenAiContext::create_new_buf();
    let mut record_write_buf = OpenAiContext::create_new_buf();
    let openai_context =
        OpenAiContext::new(&mut resolver, &mut record_read_buf, &mut record_write_buf)
            .expect("failed to create openai context");
    let mut openai = OpenAi::new(OPENAI_API_KEY, openai_context).expect("failed to create openai");

    psp::dprintln!("Created openai context!");

    openai.ask_gpt("Hello, world!").expect("failed to ask gpt");

    let read_text = unsafe {
        sceKernelDcacheWritebackAll();

        let mut out_text: Vec<u16> = Vec::with_capacity(CHAT_MAX_LENGTH_USIZE);
        let out_capacity: i32 = out_text.capacity() as i32;

        let description = str_to_u16_mut_ptr("Ask GPT\0");
        let mut osk_data = default_osk_data(description, out_capacity, out_text.as_mut_ptr());

        let params = &mut default_osk_params(&mut osk_data);

        psp::dprintln!("starting osk...");

        setup_gu();

        start_osk(params).expect("failed to start osk");

        read_from_osk(params).unwrap_or_default()
    };

    psp::dprintln!("read_text: {:?}", read_text);

    openai
        .ask_gpt(read_text.as_str())
        .expect("failed to ask gpt");

    unsafe {
        sceGuTerm();
        sceKernelExitGame();
    };
}
