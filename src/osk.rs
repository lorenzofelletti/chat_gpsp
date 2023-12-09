use core::{ffi::c_void, mem::size_of, sync::atomic::AtomicBool};

use alloc::{borrow::ToOwned, string::String, vec, vec::Vec};
use lazy_static::lazy_static;
use psp::{
    dprint,
    sys::{
        self, sceDisplayWaitVblankStart, sceGuClear, sceGuClearColor, sceGuClearDepth,
        sceGuDebugPrint, sceGuDepthBuffer, sceGuDepthFunc, sceGuDepthRange, sceGuDispBuffer,
        sceGuDisplay, sceGuDrawBuffer, sceGuEnable, sceGuFinish, sceGuFrontFace, sceGuInit,
        sceGuOffset, sceGuScissor, sceGuShadeModel, sceGuStart, sceGuSwapBuffers, sceGuSync,
        sceGuTerm, sceGuViewport, sceKernelDelayThread, sceKernelExitGame,
        sceUtilityGetSystemParamInt, sceUtilityOskGetStatus, sceUtilityOskInitStart,
        sceUtilityOskShutdownStart, sceUtilityOskUpdate, ClearBuffer, DisplayPixelFormat, GuState,
        GuSyncMode, LightComponent, SceUtilityOskData, SceUtilityOskInputLanguage,
        SceUtilityOskInputType, SceUtilityOskParams, SceUtilityOskState, TexturePixelFormat,
        ThreadAttributes, UtilityDialogCommon, UtilityMsgDialogParams,
    },
    vram_alloc::get_vram_allocator,
};

static mut LIST: psp::Align16<[u32; 262144]> = psp::Align16([0; 262144]);
const SCR_WIDTH: i32 = 480;
const SCR_HEIGHT: i32 = 272;
const BUF_WIDTH: i32 = 512;
const SCR_WIDTH_U32: u32 = SCR_WIDTH as u32;
const SCR_HEIGHT_U32: u32 = SCR_HEIGHT as u32;
const BUF_WIDTH_U32: u32 = BUF_WIDTH as u32;

const CHAT_MAX_LENGTH: u16 = 32;
const CHAT_MAX_LENGTH_USIZE: usize = CHAT_MAX_LENGTH as usize;

// --- START FROM SAMPLE ---
lazy_static! {
    static ref DONE: AtomicBool = AtomicBool::new(false);
}

extern "C" fn exit_callback(_arg1: i32, _arg2: i32, _common: *mut c_void) -> i32 {
    DONE.store(true, core::sync::atomic::Ordering::SeqCst);
    0
}

extern "C" fn callback_thread(_args: usize, _argp: *mut c_void) -> i32 {
    let cbid =
        unsafe { sys::sceKernelCreateCallback([].as_ptr(), exit_callback, 0 as *mut c_void) };
    unsafe { sys::sceKernelRegisterExitCallback(cbid) };
    unsafe { sys::sceKernelSleepThreadCB() };
    0
}

fn setup_callbacks() -> i32 {
    let thid = unsafe {
        sys::sceKernelCreateThread(
            b"update_thread\0".as_ptr() as *const _,
            callback_thread,
            0x11,
            0xFA0,
            ThreadAttributes::USER,
            core::ptr::null_mut(),
        )
    };
    if thid.0 >= 0 {
        unsafe { sys::sceKernelStartThread(thid, 0, core::ptr::null_mut()) };
    }
    thid.0
}

pub fn main_fn() {
    setup_callbacks();

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

        let mut out_text = [0u16; CHAT_MAX_LENGTH_USIZE];
        let mut description = "ask GPT".to_owned();
        let max_text_length = CHAT_MAX_LENGTH;

        let mut osk_data = get_osk_data(&mut description, max_text_length, &mut out_text);

        let params = &mut default_utility_osk_params(&mut osk_data);

        start_osk(params).expect("failed to start osk");

        let read_text = read_from_osk(params).unwrap_or_default();

        while !DONE.load(core::sync::atomic::Ordering::SeqCst) {
            sceKernelDelayThread(1_000);
        }

        let allocator = get_vram_allocator().unwrap();
        let fbp0 = allocator
            .alloc_texture_pixels(BUF_WIDTH_U32, SCR_HEIGHT_U32, TexturePixelFormat::Psm8888)
            .as_mut_ptr_from_zero();

        //sys::sceGuInit();
        /*sys::sceGuStart(
            sys::GuContextType::Direct,
            &mut LIST as *mut _ as *mut c_void,
        );*/
        sceGuDebugPrint(100, 100, 0xff0000ff, read_text.as_bytes().as_ptr());
        sys::sceGuDrawBuffer(DisplayPixelFormat::Psm8888, fbp0 as _, BUF_WIDTH as i32);
        sys::sceGuDebugPrint(100, 100, 0xff0000ff, b"Hello World\0" as *const u8);
        sys::sceGuDebugFlush();

        sys::sceGuFinish();
        sys::sceGuSync(sys::GuSyncMode::Finish, sys::GuSyncBehavior::Wait);
        sys::sceDisplayWaitVblankStart();
        sys::sceGuDisplay(true);

        psp::dprintln!("read text: '{:?}'", read_text);
        sceGuTerm();

        sceKernelExitGame();
    }
}

// --- END FROM SAMPLE ---

unsafe fn unsafe_setup_gu() {
    sys::sceGuInit();
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

pub fn setup_gu() {
    unsafe {
        unsafe_setup_gu();
    }
}

/// Convert a mutable pointer to a u16 to a Vec<u16>.
fn mut_ptr_u16_to_vec_u16(ptr: *mut u16, len: usize) -> Vec<u16> {
    let mut vec = vec![0u16; len];
    for (i, c) in vec.iter_mut().enumerate().take(len) {
        *c = unsafe { ptr.add(i) } as u16;
    }
    vec
}

pub fn get_osk_data(
    description: &mut str,
    max_text_length: u16,
    out_text: &mut [u16],
) -> SceUtilityOskData {
    let mut msg: [u8; 512] = [0u8; 512];
    msg[..description.len()].copy_from_slice(description.as_bytes());
    let mut in_text = [0u16; 0];
    SceUtilityOskData {
        unk_00: 0,
        unk_04: 0,
        language: SceUtilityOskInputLanguage::English,
        unk_12: 0,
        inputtype: SceUtilityOskInputType::All,
        lines: 1,
        unk_24: 1,
        desc: [].as_mut_ptr(), // description.as_mut_ptr() as *mut u16,
        intext: in_text.as_mut_ptr(),
        outtextlength: max_text_length.into(),
        outtext: out_text.as_mut_ptr(),
        result: sys::SceUtilityOskResult::Unchanged,
        outtextlimit: max_text_length.into(),
    }
}

pub fn default_utility_osk_params(data: &mut SceUtilityOskData) -> SceUtilityOskParams {
    SceUtilityOskParams {
        base: UtilityDialogCommon {
            // size of data
            size: size_of::<SceUtilityOskParams>() as u32,
            language: sys::SystemParamLanguage::English,
            button_accept: sys::UtilityDialogButtonAccept::Cross,
            graphics_thread: 0x11, //17,
            access_thread: 0x13,   //19,
            font_thread: 0x12,     //18,
            sound_thread: 0x10,    //16,
            result: 0,             //i32::default(),
            reserved: [0i32; 4],
        },
        datacount: 1,
        data: data,
        state: sys::SceUtilityOskState::Visible, //None,
        unk_60: 0,
    }
}

pub fn start_osk(params: &mut SceUtilityOskParams) -> Result<(), &str> {
    setup_gu();
    unsafe {
        if sceUtilityOskInitStart(params as *mut SceUtilityOskParams) == 0 {
            Ok(())
        } else {
            Err("cannot init osk")
        }
    }
}

struct OskState {
    state: SceUtilityOskState,
}

impl OskState {
    #[allow(dead_code)]
    #[inline]
    pub fn new(state: SceUtilityOskState) -> Self {
        Self { state }
    }

    #[inline]
    pub fn default() -> Self {
        Self {
            state: SceUtilityOskState::None,
        }
    }

    #[inline]
    pub fn get(&self) -> SceUtilityOskState {
        self.state
    }

    #[allow(dead_code)]
    #[inline]
    pub fn get_mut(&mut self) -> &mut SceUtilityOskState {
        &mut self.state
    }

    /// Set the state of the OSK to the given value.
    #[inline]
    pub fn set(&mut self, state: i32) {
        if !(0..=5).contains(&state) {
            self.state = SceUtilityOskState::None;
        }
        self.state = unsafe { core::mem::transmute(state) };
    }
}

impl From<i32> for OskState {
    #[inline]
    fn from(state: i32) -> Self {
        Self {
            state: unsafe { core::mem::transmute(state) },
        }
    }
}

pub fn read_from_osk(params: &mut SceUtilityOskParams) -> Option<String> {
    let params = params as *mut SceUtilityOskParams;

    let mut done = false;

    let mut osk_state = OskState::default();

    //setup_gu();

    while !done {
        unsafe {
            sceGuStart(
                sys::GuContextType::Direct,
                &mut LIST as *mut _ as *mut c_void, // from rust-psp msgdialog example
            );
            sceGuClearColor(0);
            sceGuClearDepth(0);

            sceGuFinish();
            sceGuSync(GuSyncMode::Finish, sys::GuSyncBehavior::Wait);

            sceGuClear(ClearBuffer::COLOR_BUFFER_BIT);

            osk_state.set(sceUtilityOskGetStatus());

            psp::dprintln!("osk state: {:?}", osk_state.get());

            match osk_state.get() {
                SceUtilityOskState::None => done = true,
                SceUtilityOskState::Visible => {
                    //sceUtilityOskUpdate(1);
                    sceUtilityOskShutdownStart();
                }
                SceUtilityOskState::Quit => {
                    //sceUtilityOskShutdownStart();
                    done = true;
                }
                SceUtilityOskState::Finished => {
                    //sceUtilityOskShutdownStart();
                    done = true;
                }
                SceUtilityOskState::Initialized => {
                    sceUtilityOskUpdate(1);
                }
                _ => {}
            }

            //pspDebugScreenInit();
            //pspDebugScreenSetXY(0, 0);

            //sceDisplayWaitVblankStart();
            //sceGuSwapBuffers();
        };

        DONE.store(true, core::sync::atomic::Ordering::SeqCst);
    }

    let osk_data: SceUtilityOskData = unsafe { *(*params).data };
    // get osk_data.result
    match osk_data.result {
        sys::SceUtilityOskResult::Cancelled => None,
        _ => {
            let out_text =
                mut_ptr_u16_to_vec_u16(osk_data.outtext, osk_data.outtextlength as usize);
            Some(String::from_utf16(&out_text).unwrap())
        }
    }
}
