use core::{ffi::c_void, mem::size_of, sync::atomic::AtomicBool};

use alloc::{string::String, vec, vec::Vec};
use lazy_static::lazy_static;
use psp::{
    dprint,
    sys::{
        self, sceDisplayWaitVblankStart, sceGuClear, sceGuClearColor, sceGuClearDepth,
        sceGuDebugPrint, sceGuDepthBuffer, sceGuDepthFunc, sceGuDepthRange, sceGuDispBuffer,
        sceGuDisplay, sceGuDrawBuffer, sceGuEnable, sceGuFinish, sceGuFrontFace, sceGuInit,
        sceGuOffset, sceGuScissor, sceGuShadeModel, sceGuStart, sceGuSync, sceGuViewport,
        sceKernelDelayThread, sceUtilityOskGetStatus, sceUtilityOskInitStart,
        sceUtilityOskShutdownStart, sceUtilityOskUpdate, ClearBuffer, GuState, GuSyncMode,
        SceUtilityOskData, SceUtilityOskInputLanguage, SceUtilityOskInputType, SceUtilityOskParams,
        SceUtilityOskState, UtilityDialogCommon,
    },
};

static mut LIST: psp::Align16<[u32; 262144]> = psp::Align16([0; 262144]);

const SCR_WIDTH: i32 = 480;
const SCR_HEIGHT: i32 = 272;
const BUF_WIDTH: i32 = 512;
#[allow(dead_code)]
const SCR_WIDTH_U32: u32 = SCR_WIDTH as u32;
#[allow(dead_code)]
const SCR_HEIGHT_U32: u32 = SCR_HEIGHT as u32;
#[allow(dead_code)]
const BUF_WIDTH_U32: u32 = BUF_WIDTH as u32;

const CHAT_MAX_LENGTH: u16 = 32;
const CHAT_MAX_LENGTH_USIZE: usize = CHAT_MAX_LENGTH as usize;

// --- START FROM SAMPLE ---
lazy_static! {
    static ref DONE: AtomicBool = AtomicBool::new(false);
}

fn str_to_u16_mut_ptr(s: &str) -> *mut u16 {
    let mut vec = vec![0u16; s.len()];
    for (i, c) in s.char_indices() {
        vec[i] = c as u16;
    }
    vec.as_mut_ptr()
}

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

        let mut out_text = [0u16; CHAT_MAX_LENGTH_USIZE];

        let description = str_to_u16_mut_ptr("Ask GPT");

        let max_text_length = CHAT_MAX_LENGTH;

        let mut osk_data = get_osk_data(description, max_text_length, &mut out_text);

        let params = &mut default_utility_osk_params(&mut osk_data);

        start_osk(params).expect("failed to start osk");

        let read_text = read_from_osk(params).unwrap_or_default();

        while !DONE.load(core::sync::atomic::Ordering::SeqCst) {
            sceKernelDelayThread(100_000); // 100ms
        }

        sys::sceKernelDcacheWritebackInvalidateAll();

        psp::dprintln!("read text: '{:?}'", read_text);
    }
}

fn setup_gu() {
    unsafe {
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
    description: *mut u16,
    max_text_length: u16,
    out_text: &mut [u16],
) -> SceUtilityOskData {
    let mut in_text = [0u16; CHAT_MAX_LENGTH_USIZE];

    SceUtilityOskData {
        unk_00: 0,
        unk_04: 0,
        language: SceUtilityOskInputLanguage::English,
        unk_12: 0,
        inputtype: SceUtilityOskInputType::All,
        lines: 1,
        unk_24: 1,
        desc: description, // description.as_mut_ptr() as *mut u16,
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

    // somehow otherwise the osk won't show up
    dprint!("");

    while !done {
        unsafe {
            osk_state.set(sceUtilityOskGetStatus());

            // the higher, the slower the osk will be
            sceKernelDelayThread(16_000);

            match osk_state.get() {
                SceUtilityOskState::None => done = true,
                SceUtilityOskState::Visible => {
                    let res = sceUtilityOskShutdownStart();
                    if res < 0 {
                        panic!("cannot shutdown osk: {}", res);
                    }
                }
                SceUtilityOskState::Quit => {
                    done = true;
                }
                SceUtilityOskState::Finished => {
                    done = true;
                }
                SceUtilityOskState::Initialized => {
                    sceUtilityOskUpdate(1);
                }
                _ => {}
            }
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
