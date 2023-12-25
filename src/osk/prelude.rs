use core::mem::size_of;

use psp::sys::{
    self, SceUtilityOskData, SceUtilityOskInputLanguage, SceUtilityOskInputType,
    SceUtilityOskParams, UtilityDialogCommon,
};

use crate::utils::str_to_u16_mut_ptr;

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
        outtextlimit: max_text_length.into(),
    }
}

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
        data: data,
        state: sys::SceUtilityOskState::None,
        unk_60: 0,
    }
}
