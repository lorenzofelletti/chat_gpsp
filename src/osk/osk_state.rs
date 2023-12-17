use psp::sys::{sceUtilityOskGetStatus, SceUtilityOskState};

/// The state of the osk.
pub struct OskState {
    state: SceUtilityOskState,
}

impl OskState {
    /// Create a new OskState.
    #[allow(dead_code)]
    #[inline]
    pub fn new() -> Self {
        Self {
            state: unsafe { core::mem::transmute(sceUtilityOskGetStatus()) },
        }
    }

    /// Get the current state of the osk.
    #[inline]
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
