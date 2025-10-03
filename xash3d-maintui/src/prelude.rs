pub use xash3d_ui::prelude::*;

pub fn engine() -> UiEngineRef {
    // SAFETY: we do not use threads
    unsafe { UiEngineRef::new() }
}
