use core::{ffi::c_void, mem::size_of};

use alloc::{string::String, vec, vec::Vec};
use psp::{
    dprint,
    sys::{
        self, sceDisplayWaitVblankStart, sceGuClear, sceGuFinish, sceGuStart, sceGuSwapBuffers,
        sceGuSync, sceUtilityGetSystemParamInt, sceUtilityOskGetStatus, sceUtilityOskInitStart,
        sceUtilityOskShutdownStart, sceUtilityOskUpdate, ClearBuffer, GuSyncMode,
        SceUtilityOskData, SceUtilityOskInputLanguage, SceUtilityOskInputType, SceUtilityOskParams,
        SceUtilityOskState, UtilityDialogCommon,
    },
};

/// Convert a mutable pointer to a u16 to a Vec<u16>.
fn mut_ptr_u16_to_vec_u16(ptr: *mut u16, len: usize) -> Vec<u16> {
    let mut vec = vec![0u16; len];
    for (i, c) in vec.iter_mut().enumerate().take(len) {
        *c = unsafe { ptr.add(i) } as u16;
    }
    vec
}

pub fn initialize_osk_data_old(
    osk_data: &mut SceUtilityOskData,
    description: &mut str,
    max_text_length: u16,
    out_text: &mut [u16],
) {
    osk_data.language = SceUtilityOskInputLanguage::English;
    osk_data.desc = description.as_mut_ptr() as *mut u16;

    osk_data.lines = 1;
    osk_data.outtextlength = max_text_length.into();
    osk_data.outtextlimit = max_text_length.into();
    osk_data.outtext = out_text.as_mut_ptr();

    // unknown, pass 0
    osk_data.unk_00 = 0;
    osk_data.unk_04 = 0;
    osk_data.unk_12 = 0;
    osk_data.unk_24 = 0;
}

pub fn get_osk_data(
    description: &mut str,
    max_text_length: u16,
    out_text: &mut [u16],
) -> SceUtilityOskData {
    let mut in_text = [0u16; 0];
    SceUtilityOskData {
        unk_00: 0,
        unk_04: 0,
        language: SceUtilityOskInputLanguage::English,
        unk_12: 0,
        inputtype: SceUtilityOskInputType::All,
        lines: 1,
        unk_24: 0,
        desc: description.as_mut_ptr() as *mut u16,
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
            graphics_thread: 17,
            access_thread: 19,
            font_thread: 18,
            sound_thread: 16,
            result: i32::default(),
            reserved: [0, 0, 0, 0],
        },
        datacount: 1,
        data: data,
        state: sys::SceUtilityOskState::None,
        unk_60: 0,
    }
}

pub fn start_osk(params: &mut SceUtilityOskParams) -> Result<(), &str> {
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

    while !done {
        unsafe {
            // list: 16 bytes aligned
            let list = vec![0u8, 16].as_mut_ptr() as *mut c_void;
            sceGuStart(sys::GuContextType::Direct, list);
            sceGuClear(ClearBuffer::COLOR_BUFFER_BIT);
            sceGuFinish();

            sceGuSync(GuSyncMode::Finish, sys::GuSyncBehavior::Wait);

            sceGuClear(ClearBuffer::COLOR_BUFFER_BIT);

            osk_state.set(sceUtilityOskGetStatus());

            match osk_state.get() {
                SceUtilityOskState::None => done = true,
                SceUtilityOskState::Visible => {
                    sceUtilityOskUpdate(1);
                }
                SceUtilityOskState::Quit => {
                    sceUtilityOskShutdownStart();
                }
                // non la metto mai a visible dio caro!!!
                _ => {}
            }

            sceDisplayWaitVblankStart();
            sceGuSwapBuffers();
        };
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
