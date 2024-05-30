use psp::sys::{sceUtilityOskGetStatus, PspUtilityDialogState};

/// The state of the osk.
pub struct OskState {
    state: PspUtilityDialogState,
}

impl OskState {
    #[allow(dead_code)]
    #[inline]
    /// Create a new OskState.
    pub fn new() -> Self {
        Self {
            state: unsafe {
                core::mem::transmute::<i32, psp::sys::PspUtilityDialogState>(
                    sceUtilityOskGetStatus(),
                )
            },
        }
    }

    #[inline]
    /// Get the current state of the osk.
    pub fn get(&mut self) -> PspUtilityDialogState {
        self.state = unsafe {
            core::mem::transmute::<i32, psp::sys::PspUtilityDialogState>(sceUtilityOskGetStatus())
        };
        self.state
    }
}

impl From<i32> for OskState {
    #[inline]
    fn from(state: i32) -> Self {
        Self {
            state: unsafe { core::mem::transmute::<i32, psp::sys::PspUtilityDialogState>(state) },
        }
    }
}
