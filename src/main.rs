#![no_std]
#![no_main]
#![feature(c_void_variant)]

extern crate alloc;

use alloc::borrow::ToOwned;
use osk::{main_fn, read_from_osk, start_osk};
use psp::sys::{
    self, SceUtilityOskData, SceUtilityOskInputLanguage, SceUtilityOskInputType,
    SceUtilityOskParams,
};

use drogue_network::addr::HostSocketAddr;
use drogue_tls::blocking::*;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha20Rng;

use crate::osk::get_osk_data;

psp::module!("tls-test", 1, 1);

mod net;
mod osk;

fn select_netconfig() -> i32 {
    unsafe { psp::sys::sceUtilityCheckNetParam(1) }
}

const CHAT_MAX_LENGTH: u16 = 32;
const CHAT_MAX_LENGTH_USIZE: usize = CHAT_MAX_LENGTH as usize;

#[no_mangle]
fn psp_main() {
    psp::enable_home_button();

    main_fn();
    /*unsafe {
        load_modules();
        init();

        let conn_number = 0;

        psp::sys::sceNetApctlConnect(2);
        psp::dprintln!("Connecting...");
        loop {
            let mut state: psp::sys::ApctlState = core::mem::zeroed();
            psp::sys::sceNetApctlGetState(&mut state);
            match state {
                sys::ApctlState::Disconnected => psp::dprintln!("X"),
                sys::ApctlState::Scanning => psp::dprintln!("SKAN"),
                sys::ApctlState::Joining => psp::dprintln!("Gioining"),
                sys::ApctlState::GettingIp => psp::dprintln!("ghetting ai pee"),
                sys::ApctlState::GotIp => {
                    psp::dprintln!("Orco dio");
                    break;
                }
                sys::ApctlState::EapAuth => psp::dprintln!("Y"),
                sys::ApctlState::KeyExchange => psp::dprintln!("Z"),
            }
            psp::sys::sceKernelDelayThread(500_000);
        }
    }*/

    /*
    let mut out_text = [0u16; CHAT_MAX_LENGTH_USIZE];
    let mut description = "ask GPT".to_owned();
    let max_text_length = CHAT_MAX_LENGTH;

    let mut osk_data = get_osk_data(&mut description, max_text_length, &mut out_text);

    let params = &mut osk::default_utility_osk_params(&mut osk_data);

    start_osk(params).expect("failed to start osk");

    let read_text = read_from_osk(params).unwrap_or_default();
    psp::dprintln!("read text: '{:?}'", read_text);*/

    /*
    let socket = net::Socket::open().expect("failed to open socket");
    socket
        .connect(HostSocketAddr::from("93.184.216.34", 443).expect("failed to create address"))
        .expect("failed to connect socket");
    let mut seed: u64 = 0;
    unsafe {
        sys::sceRtcGetCurrentTick(&mut seed as *mut u64);
    }
    let rng = ChaCha20Rng::seed_from_u64(seed);

    let mut record_buffer = [0; 16384];
    let tls_context = TlsContext::new(rng, &mut record_buffer).with_server_name("example.com");
    let mut tls: TlsConnection<ChaCha20Rng, net::Socket, Aes128GcmSha256> =
        TlsConnection::new(tls_context, socket);

    tls.open().expect("error establishing TLS connection");

    tls.write(b"GET / HTTP/1.1\r\nHost: www.example.com\r\nUser-Agent: A fucking PSP!\r\n\r\n")
        .expect("error writing data");

    let mut rx_buf = [0; 4096];
    let sz = tls.read(&mut rx_buf).expect("error reading data");
    unsafe {
        let mut text = alloc::string::String::from_utf8_unchecked(rx_buf.to_vec());
        text = text.replace("\r", "");
        text = text.replace("\0", "");
        psp::dprintln!("Read {} bytes: {}", sz, text);
    }

    let mut rx_buf = [0; 4096];
    let sz = tls.read(&mut rx_buf).expect("error reading data");
    unsafe {
        let mut text = alloc::string::String::from_utf8_unchecked(rx_buf.to_vec());
        text = text.replace("\r", "");
        text = text.replace("\0", "");
        psp::dprintln!("Read {} bytes: {}", sz, text);
    }*/
}

unsafe fn load_modules() {
    psp::sys::sceUtilityLoadNetModule(psp::sys::NetModule::NetCommon);
    psp::sys::sceUtilityLoadNetModule(psp::sys::NetModule::NetInet);
}

unsafe fn init() {
    psp::sys::sceNetInit(0x20000, 0x20, 0x1000, 0x20, 0x1000);
    psp::sys::sceNetInetInit();
    psp::sys::sceNetResolverInit();
    psp::sys::sceNetApctlInit(0x1600, 42);
}
