use core::ffi::c_void;

use alloc::{string::String, vec::Vec};

use psp::{
    dprint,
    sys::{
        self, sceDisplayWaitVblankStart, sceGuClear, sceGuDebugFlush, sceGuDebugPrint,
        sceGuDepthBuffer, sceGuDepthFunc, sceGuDepthRange, sceGuDispBuffer, sceGuDisplay,
        sceGuDrawBuffer, sceGuEnable, sceGuFinish, sceGuFrontFace, sceGuInit, sceGuOffset,
        sceGuScissor, sceGuShadeModel, sceGuStart, sceGuSwapBuffers, sceGuSync, sceGuTerm,
        sceGuViewport, sceKernelDcacheWritebackAll, sceKernelDcacheWritebackInvalidateAll,
        sceUtilityOskInitStart, sceUtilityOskShutdownStart, sceUtilityOskUpdate, ClearBuffer,
        DisplayPixelFormat, GuState, GuSyncBehavior, GuSyncMode, SceUtilityOskData,
        SceUtilityOskParams, SceUtilityOskState, TexturePixelFormat,
    },
    vram_alloc::get_vram_allocator,
};

use crate::osk::utils::*;
use crate::osk::{constants::*, prelude::default_osk_data};
use crate::osk::{osk_state::OskState, prelude::default_osk_params};

pub mod constants;
pub mod osk_state;
pub mod prelude;
pub mod utils;

static mut LIST: psp::Align16<[u32; 262144]> = psp::Align16([0; 262144]);

pub fn main_fn() {
    unsafe {
        sceGuInit();
        sceGuStart(
            sys::GuContextType::Direct,
            &mut LIST as *mut _ as *mut c_void,
        );
        sceGuDrawBuffer(
            sys::DisplayPixelFormat::Psm8888,
            0 as *mut c_void,
            BUF_WIDTH,
        );
        sceGuDispBuffer(SCR_WIDTH, SCR_HEIGHT, 0x88000 as *mut c_void, BUF_WIDTH);
        sceGuDepthBuffer(0x110000 as *mut c_void, BUF_WIDTH);
        sceGuOffset(
            2048 - (SCR_WIDTH as u32 / 2),
            2048 - (SCR_HEIGHT as u32 / 2),
        );
        sceGuViewport(2048, 2048, SCR_WIDTH, SCR_HEIGHT);
        sceGuDepthRange(0xc350, 0x2710);
        sceGuScissor(0, 0, SCR_WIDTH, SCR_HEIGHT);
        sceGuEnable(GuState::ScissorTest);
        sceGuDepthFunc(sys::DepthFunc::GreaterOrEqual);
        sceGuEnable(GuState::DepthTest);
        sceGuFrontFace(sys::FrontFaceDirection::Clockwise);
        sceGuShadeModel(sys::ShadingModel::Flat);
        sceGuEnable(GuState::CullFace);
        sceGuEnable(GuState::Texture2D);
        sceGuEnable(GuState::ClipPlanes);
        sceGuFinish();
        sceGuSync(GuSyncMode::Finish, sys::GuSyncBehavior::Wait);
        sceDisplayWaitVblankStart();
        sceGuDisplay(true);

        let mut out_text = Vec::with_capacity(CHAT_MAX_LENGTH_USIZE);
        let mut out_text = out_text.as_mut_slice();

        let description = str_to_u16_mut_ptr("Ask GPT\0");

        let max_text_length = CHAT_MAX_LENGTH;

        let mut osk_data = default_osk_data(description, max_text_length, &mut out_text);

        let params = &mut default_osk_params(&mut osk_data);

        start_osk(params).expect("failed to start osk");

        let read_text = read_from_osk(params).unwrap_or_default();
        //assert_eq!(2, 1);

        sceKernelDcacheWritebackAll();
        //sys::sceKernelDcacheWritebackInvalidateAll();

        sceGuFinish();
        sceGuSync(GuSyncMode::Finish, sys::GuSyncBehavior::Wait);

        psp::dprintln!("read text: '{:?}'", read_text);
    }
}

fn setup_gu() {
    unsafe {
        //sys::sceGuInit();
        sys::sceGuStart(
            sys::GuContextType::Direct,
            &mut LIST as *mut _ as *mut c_void,
        );
        sys::sceGuDrawBuffer(
            sys::DisplayPixelFormat::Psm8888,
            core::ptr::null_mut(),
            BUF_WIDTH,
        );
        sys::sceGuDispBuffer(SCR_WIDTH, SCR_HEIGHT, 0x88000 as *mut c_void, BUF_WIDTH);
        sys::sceGuDepthBuffer(0x110000 as *mut c_void, BUF_WIDTH);
        sys::sceGuOffset(
            2048 - (SCR_WIDTH as u32 / 2),
            2048 - (SCR_HEIGHT as u32 / 2),
        );
        sys::sceGuViewport(2048, 2048, SCR_WIDTH, SCR_HEIGHT);
        sys::sceGuDepthRange(0xc350, 0x2710);
        sys::sceGuScissor(0, 0, SCR_WIDTH, SCR_HEIGHT);
        sys::sceGuEnable(sys::GuState::ScissorTest);
        sys::sceGuDepthFunc(sys::DepthFunc::GreaterOrEqual);
        sys::sceGuEnable(sys::GuState::DepthTest);
        sys::sceGuFrontFace(sys::FrontFaceDirection::Clockwise);
        sys::sceGuShadeModel(sys::ShadingModel::Smooth);
        sys::sceGuEnable(sys::GuState::CullFace);
        sys::sceGuEnable(sys::GuState::ClipPlanes);
        sys::sceGuFinish();
        sys::sceGuSync(GuSyncMode::Finish, sys::GuSyncBehavior::Wait);

        sys::sceDisplayWaitVblankStart();
        sys::sceGuDisplay(true);
    }
}

#[inline]
pub fn start_osk(params: &mut SceUtilityOskParams) -> Result<(), &str> {
    //setup_gu();
    unsafe {
        if sceUtilityOskInitStart(params as *mut SceUtilityOskParams) == 0 {
            Ok(())
        } else {
            Err("cannot init osk")
        }
    }
}

pub fn read_from_osk(params: &mut SceUtilityOskParams) -> Option<String> {
    let params = params as *mut SceUtilityOskParams;

    let mut done = false;

    let mut osk_state = OskState::new();

    unsafe {
        while !done {
            sceGuStart(
                sys::GuContextType::Direct,
                &mut LIST as *mut _ as *mut c_void,
            );
            sceGuClear(ClearBuffer::COLOR_BUFFER_BIT);
            sceGuFinish();
            sceGuSync(GuSyncMode::Finish, GuSyncBehavior::Wait);

            sceGuClear(ClearBuffer::COLOR_BUFFER_BIT);

            match osk_state.get() {
                SceUtilityOskState::None => done = true,
                SceUtilityOskState::Initialized => {
                    if sceUtilityOskUpdate(1).is_negative() {
                        panic!("cannot update osk");
                    }
                }
                SceUtilityOskState::Visible => {
                    if sceUtilityOskShutdownStart().is_negative() {
                        panic!("cannot shutdown osk");
                    }
                }
                _ => psp::dprintln!("osk state: {:?}", osk_state.get()),
            }
            sceDisplayWaitVblankStart();
            sceGuSwapBuffers();
        }
    }
    unsafe { LIST = psp::Align16([0; 262144]) };
    x();
    //unsafe { sceKernelDcacheWritebackInvalidateAll() };
    /*
        unsafe {
            //sceGuTerm();
            //psp::dprint!("sceGuTerm() called");
            sceGuInit();
            loop {
                sceGuStart(
                    sys::GuContextType::Direct,
                    &mut LIST as *mut _ as *mut c_void,
                );
                sceGuClear(ClearBuffer::COLOR_BUFFER_BIT);

                sceGuDebugPrint(100, 100, 0xff0000ff, b"Hello World\0" as *const u8);
                sceGuDebugFlush();

                sceGuFinish();
                sceGuSync(GuSyncMode::Finish, GuSyncBehavior::Wait);

                sceDisplayWaitVblankStart();
                sceGuSwapBuffers();
                sys::sceGuDisplay(true);
            }
        }
    */
    let osk_params = unsafe { params.as_ref().unwrap() };

    let osk_data: &SceUtilityOskData = unsafe { osk_params.data.as_ref().unwrap() };
    //assert!(
    //    matches!(osk_data.result, sys::SceUtilityOskResult::Cancelled),
    //    "Result is not cancelled"
    //);
    // get osk_data.result
    match osk_data.result {
        sys::SceUtilityOskResult::Cancelled => None,
        _ => {
            let out_text =
                mut_ptr_u16_to_vec_char(osk_data.outtext, osk_data.outtextlength as usize);

            None

            //assert_eq!(2, 1);
            //x();
            //let mut out_text = String::new();
            //assert_eq!(2, 1);
            //Some("out_text".to_owned())
        }
    }
}

#[allow(unused)]
pub fn x() {
    let mut allocator = get_vram_allocator().unwrap();
    let fbp0 = allocator
        .alloc_texture_pixels(BUF_WIDTH_U32, SCR_HEIGHT_U32, TexturePixelFormat::Psm8888)
        .as_mut_ptr_from_zero();

    unsafe {
        //sys::sceGuInit();
        sys::sceGuStart(
            sys::GuContextType::Direct,
            &mut LIST as *mut _ as *mut c_void,
        );
        sys::sceGuDrawBuffer(DisplayPixelFormat::Psm8888, fbp0 as _, BUF_WIDTH);
        sys::sceGuDebugPrint(100, 100, 0xff0000ff, b"Hello World\0" as *const u8);
        sys::sceGuDebugFlush();

        sys::sceGuFinish();
        sys::sceGuSync(sys::GuSyncMode::Finish, sys::GuSyncBehavior::Wait);
        sys::sceDisplayWaitVblankStart();
        sys::sceGuDisplay(true);
    }
}
