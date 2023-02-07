#![no_std]
#![no_main]

use core::mem::MaybeUninit;

psp::module!("sample_module", 1, 1);

fn psp_main() {
    psp::enable_home_button();

    psp::dprintln!("Let's go!");

    unsafe {
        let mut tick = 0;
        psp::sys::sceRtcGetCurrentTick(&mut tick);

        let mut date = MaybeUninit::uninit();
        psp::sys::sceRtcSetTick(date.as_mut_ptr(), &tick);
        let date = date.assume_init();

        psp::dprintln!(
            "Current Date is {:02}:{:02}:{:02}",
            date.hour,
            date.minutes,
            date.seconds
        );
    }
}
