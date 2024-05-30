use core::mem::size_of;

use psp::sys::{
    self, SceUtilityOskData, SceUtilityOskInputLanguage, SceUtilityOskInputType,
    SceUtilityOskParams, UtilityDialogCommon,
};

use crate::utils::str_to_u16_mut_ptr;

#[inline]
/// Create a [`SceUtilityOskData`] with default values.
/// By default, the osk will be in English, the initial text will be empty, and the description and
/// max text length will be the ones provided.
///
/// # Parameters
/// * `description` - A mutable pointer to a u16 array containing the description of the osk.
/// * `max_text_length` - The maximum length of the text that can be entered into the osk.
/// * `out_text` - A mutable pointer to a u16 array that will contain the text entered into the osk.
///
/// # Returns
/// A [`SceUtilityOskData`].
///
/// # Safety
/// Please ensure that `description` and `out_text` are valid pointers, and that `max_text_length`
/// is correct. To be correct, `max_text_length` must be the length of the array `out_text` points to.
pub fn default_osk_data(
    description: *mut u16,
    max_text_length: i32,
    out_text: *mut u16,
) -> SceUtilityOskData {
    let in_text = str_to_u16_mut_ptr("\0");

    SceUtilityOskData {
        unk_00: 0,
        unk_04: 0,
        language: SceUtilityOskInputLanguage::English,
        unk_12: 0,
        inputtype: SceUtilityOskInputType::All,
        lines: 1,
        unk_24: 0,
        desc: description,
        intext: in_text,
        outtextlength: max_text_length,
        outtext: out_text,
        result: sys::SceUtilityOskResult::Unchanged,
        outtextlimit: max_text_length,
    }
}

#[inline]
/// Create a [`SceUtilityOskParams`] with default values.
pub fn default_osk_params(data: &mut SceUtilityOskData) -> SceUtilityOskParams {
    SceUtilityOskParams {
        base: UtilityDialogCommon {
            // size of data
            size: size_of::<SceUtilityOskParams>() as u32,
            language: sys::SystemParamLanguage::English,
            button_accept: sys::UtilityDialogButtonAccept::Cross,
            graphics_thread: 0x11,
            access_thread: 0x13,
            font_thread: 0x12,
            sound_thread: 0x10,
            result: 0,
            reserved: [0i32; 4],
        },
        datacount: 1,
        data,
        state: sys::PspUtilityDialogState::None,
        unk_60: 0,
    }
}
