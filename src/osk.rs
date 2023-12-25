use core::ffi::c_void;

use alloc::string::String;

use psp::{
    sys::{
        self, sceDisplayWaitVblankStart, sceGuAlphaFunc, sceGuClear, sceGuDepthBuffer,
        sceGuDepthFunc, sceGuDepthRange, sceGuDispBuffer, sceGuDrawBuffer, sceGuEnable,
        sceGuFinish, sceGuFrontFace, sceGuOffset, sceGuScissor, sceGuShadeModel, sceGuStart,
        sceGuSwapBuffers, sceGuSync, sceGuViewport, sceUtilityOskInitStart,
        sceUtilityOskShutdownStart, sceUtilityOskUpdate, ClearBuffer, DisplayPixelFormat, GuState,
        GuSyncBehavior, GuSyncMode, SceUtilityOskData, SceUtilityOskParams, SceUtilityOskState,
        TexturePixelFormat,
    },
    vram_alloc::get_vram_allocator,
};

use crate::osk::constants::*;
use crate::osk::osk_state::OskState;
use crate::{osk::utils::*, renderer::Renderer};

pub mod constants;
pub mod osk_state;
pub mod prelude;
pub mod utils;

static mut LIST: psp::Align16<[u32; 262_144]> = psp::Align16([0; 262_144]);

#[inline]
pub fn setup_buffers(fbp0: *mut u8, fbp1: *mut u8) {
    unsafe {
        sys::sceGuStart(
            sys::GuContextType::Direct,
            &mut LIST as *mut _ as *mut c_void,
        );
        sys::sceGuDrawBuffer(
            sys::DisplayPixelFormat::Psm8888,
            core::ptr::null_mut(),
            BUF_WIDTH,
        );
        sceGuDrawBuffer(DisplayPixelFormat::Psm8888, fbp0 as _, BUF_WIDTH);
        sceGuDispBuffer(SCR_WIDTH, SCR_HEIGHT, fbp1 as *mut c_void, BUF_WIDTH);
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
        sys::sceGuFinish();
        sys::sceGuSync(GuSyncMode::Finish, sys::GuSyncBehavior::Wait);

        sys::sceDisplayWaitVblankStart();
        sys::sceGuDisplay(true);
    }
}

pub fn setup_gu() {
    unsafe {
        sys::sceGuStart(
            sys::GuContextType::Direct,
            &mut LIST as *mut _ as *mut c_void,
        );
        sys::sceGuDrawBuffer(
            sys::DisplayPixelFormat::Psm8888,
            (BUF_WIDTH * SCR_HEIGHT * 4) as *mut c_void, // core::ptr::null_mut(),
            BUF_WIDTH,
        );
        sys::sceGuDispBuffer(SCR_WIDTH, SCR_HEIGHT, 0x0 as *mut c_void, BUF_WIDTH);
        sceGuClear(ClearBuffer::COLOR_BUFFER_BIT | ClearBuffer::DEPTH_BUFFER_BIT);
        sys::sceGuDepthBuffer(((BUF_WIDTH * SCR_HEIGHT * 4) * 2) as *mut c_void, BUF_WIDTH);
        sys::sceGuOffset(2048 - (SCR_WIDTH_U32 / 2), 2048 - (SCR_HEIGHT_U32 / 2));
        sys::sceGuViewport(2048, 2048, SCR_WIDTH, SCR_HEIGHT);
        sys::sceGuDepthRange(0xc350, 0x2710);
        sys::sceGuScissor(0, 0, SCR_WIDTH, SCR_HEIGHT);
        sys::sceGuEnable(sys::GuState::ScissorTest);
        // enable alpha test
        sys::sceGuAlphaFunc(sys::AlphaFunc::Greater, 0, 0xff);
        sys::sceGuEnable(GuState::AlphaTest);
        // enable depth test
        sys::sceGuDepthFunc(sys::DepthFunc::Greater);
        sys::sceGuEnable(sys::GuState::DepthTest);

        sys::sceGuFrontFace(sys::FrontFaceDirection::Clockwise);
        sys::sceGuShadeModel(sys::ShadingModel::Smooth);
        sys::sceGuEnable(sys::GuState::CullFace);
        // enable textures
        sys::sceGuEnable(GuState::Texture2D);
        sys::sceGuEnable(sys::GuState::ClipPlanes);
        sys::sceGuTexMode(TexturePixelFormat::Psm8888, 0, 0, 0);
        sys::sceGuTexFunc(
            sys::TextureEffect::Replace,
            sys::TextureColorComponent::Rgba,
        );
        sys::sceGuTexFilter(sys::TextureFilter::Nearest, sys::TextureFilter::Nearest);
        sys::sceGuAmbientColor(0xff_ff_ff_ff);
        sys::sceGuEnable(GuState::Blend);
        sys::sceGuBlendFunc(
            sys::BlendOp::Add,
            sys::BlendFactor::SrcAlpha,
            sys::BlendFactor::OneMinusSrcAlpha,
            0,
            0,
        );

        sys::sceGuFinish();
        sys::sceGuSync(GuSyncMode::Finish, sys::GuSyncBehavior::Wait);

        sys::sceDisplayWaitVblankStart();
        sys::sceGuDisplay(true);
    }
}

#[inline]
pub fn start_osk(params: &mut SceUtilityOskParams) -> Result<(), &str> {
    unsafe {
        if sceUtilityOskInitStart(params as *mut SceUtilityOskParams) == 0 {
            Ok(())
        } else {
            Err("cannot init osk")
        }
    }
}

pub fn read_from_osk(
    params: &mut SceUtilityOskParams,
    renderer: Option<&Renderer>,
) -> Option<String> {
    let renderer = renderer.unwrap();
    let mut done = false;
    let mut osk_state = OskState::new();

    //renderer.clear(0x00000000);

    //setup_gu();
    //unsafe { sys::sceGuDisplay(true) };

    unsafe {
        sceGuSwapBuffers();
        while !done {
            //sceGuStart(
            //    sys::GuContextType::Direct,
            //    &mut LIST as *mut _ as *mut c_void,
            //);
            //sceGuClear(ClearBuffer::COLOR_BUFFER_BIT);
            //sceGuFinish();
            //sceGuSync(GuSyncMode::Finish, GuSyncBehavior::Wait);

            //renderer.clear(0x00000000);

            //sceGuClear(ClearBuffer::COLOR_BUFFER_BIT);
            //sys::sceKernelDelayThread(16_000);

            match osk_state.get() {
                SceUtilityOskState::None => done = true,
                // should be called SceUtilityOskState::Visible
                SceUtilityOskState::Initialized => {
                    if sceUtilityOskUpdate(1).is_negative() {
                        panic!("cannot update osk");
                    }
                    //sceGuSwapBuffers();
                }
                // should be called SceUtilityOskState::Quit
                SceUtilityOskState::Visible => {
                    if sceUtilityOskShutdownStart().is_negative() {
                        panic!("cannot shutdown osk");
                    }
                }
                _ => (),
            }
            sceDisplayWaitVblankStart();
            //sceGuSwapBuffers();
        }

        sceUtilityOskShutdownStart();
        sceDisplayWaitVblankStart();
        sceGuSwapBuffers();
        sceGuSync(GuSyncMode::Finish, GuSyncBehavior::Wait);

        // sceGuStart(
        //     sys::GuContextType::Direct,
        //     &mut LIST as *mut _ as *mut c_void,
        // );
        // sceGuClear(ClearBuffer::COLOR_BUFFER_BIT);
        // sceGuFinish();
        // sceGuSync(GuSyncMode::Finish, GuSyncBehavior::Wait);
    }
    //unsafe {
    //    sceGuFinish();
    //    sceGuTerm();
    //};

    //unsafe { sceKernelDelayThread(100_000) };

    // loop {
    // psp::dprintln!("cdsvbeonskavbe");
    // }

    //unsafe { sceGuSwapBuffers() };

    //unsafe { sceKernelDcacheWritebackInvalidateAll() };

    //psp::dprintln!("finised");

    // unsafe {
    //     //sceGuFinish();
    //     //sceGuClear(ClearBuffer::COLOR_BUFFER_BIT);

    //     let done = false;
    //     while !done {
    //         sceGuStart(
    //             sys::GuContextType::Direct,
    //             &mut LIST as *mut _ as *mut c_void,
    //         );
    //         sceGuClear(ClearBuffer::COLOR_BUFFER_BIT);
    //         sceGuColor(0xff0000ff);
    //         sceGuFinish();
    //         sceGuSync(GuSyncMode::Finish, GuSyncBehavior::Wait);

    //         sceGuClear(ClearBuffer::COLOR_BUFFER_BIT);
    //         sceGuColor(0xff0000ff);

    //         sceDisplayWaitVblankStart();
    //         sceGuSwapBuffers();
    //     }
    // }
    //unsafe { LIST = psp::Align16([0; 262144]) };
    //x();
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
    //let osk_params = unsafe { params.as_ref().unwrap() };

    let osk_data: &SceUtilityOskData = unsafe { params.data.as_ref().unwrap() };
    //unsafe { osk_params.data.as_ref().unwrap() };
    //panic!("");
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

            //panic!("");

            //let x: Vec<u16> = Vec::with_capacity(30);
            //let size: i32 = x.capacity() as i32;

            //None

            //assert_eq!(2, 1);
            //x();
            let out_text = String::from_iter(out_text);
            //assert_eq!(2, 1);
            Some(out_text)
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
        sceGuSync(GuSyncMode::Finish, GuSyncBehavior::Wait);
        sys::sceDisplayWaitVblankStart();
        sceGuSwapBuffers();
        loop {
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
            sceGuSwapBuffers();
        }
        //sys::sceGuDisplay(true);
    }
}
