#[allow(dead_code)]
#[inline]
/// Load net modules
pub unsafe fn load_net_modules() {
    psp::sys::sceUtilityLoadNetModule(psp::sys::NetModule::NetCommon);
    psp::sys::sceUtilityLoadNetModule(psp::sys::NetModule::NetInet);
    psp::sys::sceUtilityLoadNetModule(psp::sys::NetModule::NetParseUri);
    psp::sys::sceUtilityLoadNetModule(psp::sys::NetModule::NetHttp);
}

#[allow(dead_code)]
#[inline]
/// Initialize net modules
pub unsafe fn net_init() {
    psp::sys::sceNetInit(0x20000, 0x20, 0x1000, 0x20, 0x1000);
    psp::sys::sceNetInetInit();
    psp::sys::sceNetResolverInit();
    psp::sys::sceNetApctlInit(0x1600, 42);
}

#[allow(dead_code)]
#[inline]
pub fn select_netconfig() -> i32 {
    unsafe { psp::sys::sceUtilityCheckNetParam(1) }
}
