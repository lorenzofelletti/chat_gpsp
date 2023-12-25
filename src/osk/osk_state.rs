use psp::sys::{sceUtilityOskGetStatus, SceUtilityOskState};

/// The state of the osk.
pub struct OskState {
    state: SceUtilityOskState,
}

impl OskState {
    #[allow(dead_code)]
    #[inline]
    /// Create a new OskState.
    pub fn new() -> Self {
        Self {
            state: unsafe { core::mem::transmute(sceUtilityOskGetStatus()) },
        }
    }

    #[inline]
    /// Get the current state of the osk.
    pub fn get(&mut self) -> SceUtilityOskState {
        self.state = unsafe { core::mem::transmute(sceUtilityOskGetStatus()) };
        self.state
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
