use core::ffi::c_void;

use alloc::string::String;

use psp::{
    sys::{
        self, sceDisplayWaitVblankStart, sceGuClear, sceGuInit, sceGuSwapBuffers,
        sceUtilityOskInitStart, sceUtilityOskShutdownStart, sceUtilityOskUpdate, ClearBuffer,
        GuState, GuSyncMode, SceUtilityOskData, SceUtilityOskParams, SceUtilityOskState,
        TexturePixelFormat,
    },
    SCREEN_HEIGHT, SCREEN_WIDTH,
};

use crate::osk::osk_state::OskState;
use crate::utils::*;

pub mod osk_state;
pub mod prelude;

static mut LIST: psp::Align16<[u32; 262_144]> = psp::Align16([0; 262_144]);

#[inline]
/// Setup GU
/// Call once to setup the Graphics Utility (GU).
pub fn setup_gu() {
    unsafe {
        sceGuInit();
        sys::sceGuStart(
            sys::GuContextType::Direct,
            &mut LIST as *mut _ as *mut c_void,
        );
        // setup buffers and viewport
        sys::sceGuDrawBuffer(
            sys::DisplayPixelFormat::Psm8888,
            (BUF_WIDTH * SCREEN_HEIGHT * 4) as *mut c_void,
            BUF_WIDTH_I32,
        );
        sys::sceGuDispBuffer(
            SCREEN_WIDTH_I32,
            SCREEN_HEIGHT_I32,
            core::ptr::null_mut(),
            BUF_WIDTH_I32,
        );
        sceGuClear(ClearBuffer::COLOR_BUFFER_BIT | ClearBuffer::DEPTH_BUFFER_BIT);
        sys::sceGuDepthBuffer(
            ((BUF_WIDTH * SCREEN_HEIGHT * 4) * 2) as *mut c_void,
            BUF_WIDTH_I32,
        );
        sys::sceGuOffset(2048 - (SCREEN_WIDTH / 2), 2048 - (SCREEN_HEIGHT / 2));
        sys::sceGuViewport(2048, 2048, SCREEN_WIDTH_I32, SCREEN_HEIGHT_I32);
        sys::sceGuDepthRange(0xc350, 0x2710);
        // setup scissor
        sys::sceGuScissor(0, 0, SCREEN_WIDTH_I32, SCREEN_HEIGHT_I32);
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
/// Initialize an OSK (On-Screen Keyboard) dialog.
/// Call once to initialize an OSK dialog.
///
/// # Parameters
/// - `params`: A mutable reference to a [`SceUtilityOskParams`] struct.
///
/// # Returns
/// - `Ok(())` if the OSK was initialized.
/// - `Err(&str)` if the OSK was not initialized.
pub fn start_osk(params: &mut SceUtilityOskParams) -> Result<(), &str> {
    unsafe {
        if sceUtilityOskInitStart(params as *mut SceUtilityOskParams) == 0 {
            Ok(())
        } else {
            Err("cannot init osk")
        }
    }
}

#[inline]
/// Read from an OSK (On-Screen Keyboard) dialog.
///
/// # Parameters
/// - `params`: A mutable reference to a [`SceUtilityOskParams`] struct.
///
/// # Returns
/// - `None` if the OSK was cancelled.
/// - `Some(String)` if the OSK was not cancelled.
///
/// # Panics
/// Panics if the OSK cannot be updated or shutdown.
pub fn read_from_osk(params: &mut SceUtilityOskParams) -> Option<String> {
    let mut done = false;
    let mut osk_state = OskState::new();

    unsafe {
        sceDisplayWaitVblankStart(); // TODO: verify - Probably not needed
        sceGuSwapBuffers(); // Probably not needed
        while !done {
            sys::sceGuStart(
                sys::GuContextType::Direct,
                &mut LIST as *mut _ as *mut c_void,
            );
            sys::sceGuClear(ClearBuffer::COLOR_BUFFER_BIT | ClearBuffer::DEPTH_BUFFER_BIT);

            sys::sceGuFinish();
            sys::sceGuSync(GuSyncMode::Finish, sys::GuSyncBehavior::Wait);

            sceGuClear(ClearBuffer::COLOR_BUFFER_BIT);
            match osk_state.get() {
                // TODO: switch to PspUtilityDialogState when it's implemented
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
                _ => (),
            }
            sceDisplayWaitVblankStart();
            sceGuSwapBuffers();
        }
    }

    let osk_data: &SceUtilityOskData = unsafe { params.data.as_ref().unwrap() };

    match osk_data.result {
        sys::SceUtilityOskResult::Cancelled => None,
        _ => {
            let out_text = unsafe {
                mut_ptr_u16_to_vec_char(osk_data.outtext, osk_data.outtextlength as usize)
            };

            let out_text = String::from_iter(out_text);

            Some(out_text)
        }
    }
}
