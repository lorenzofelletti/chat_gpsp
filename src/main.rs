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

use crate::{
    osk::setup_gu,
    utils::{str_to_u16_mut_ptr, InputHandler},
};

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

const OPENAI_API_KEY: &str = core::env!("OPENAI_API_KEY");

#[no_mangle]
fn psp_main() {
    psp::enable_home_button();

    unsafe {
        // setup network
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

        // setup controls
        psp::sys::sceCtrlSetSamplingCycle(0);
        psp::sys::sceCtrlSetSamplingMode(psp::sys::CtrlMode::Analog);
    }

    psp::dprintln!("Connected to network!");

    let mut resolver = DnsResolver::default().expect("failed to create resolver");

    let openai_context =
        OpenAiContext::new(&mut resolver, OPENAI_API_KEY).expect("failed to create openai context");

    let mut input_handler = InputHandler::default();

    psp::dprintln!("Press X to start asking GPT-3.5, any other button to exit.");
    if !input_handler.choose_continue() {
        unsafe {
            psp::dprintln!("Exiting...");
            sceGuTerm();
            sceKernelExitGame();
        };
    }

    setup_gu();

    loop {
        let read_text = unsafe {
            sceKernelDcacheWritebackAll();

            let mut out_text: Vec<u16> = Vec::with_capacity(CHAT_MAX_LENGTH_USIZE);
            let out_capacity: i32 = out_text.capacity() as i32;

            let description = str_to_u16_mut_ptr("Ask GPT\0");
            let mut osk_data = default_osk_data(description, out_capacity, out_text.as_mut_ptr());

            let params = &mut default_osk_params(&mut osk_data);

            start_osk(params).expect("failed to start osk");

            read_from_osk(params).unwrap_or_default()
        };
        let read_text = read_text.replace('\0', "");

        psp::dprintln!("User: {}\n", read_text);

        let mut openai = OpenAi::new(&openai_context).expect("failed to create openai");

        let answer = openai.ask_gpt(read_text.as_str());

        if answer.is_err() {
            psp::dprintln!("failed to get answer from openai");
            psp::dprintln!("Got error: {:?}\n", answer.err().unwrap());
        } else {
            psp::dprintln!("GPT: {}\n", answer.unwrap());
        }

        psp::dprintln!("Press X to ask again, any other button to exit.");
        if !input_handler.choose_continue() {
            break;
        }
    }

    unsafe {
        sceGuTerm();
        sceKernelExitGame();
    };
}
