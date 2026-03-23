use std::ffi::c_char;
use std::sync::Once;

static HS_INIT: Once = Once::new();

unsafe extern "C" {
    fn hs_init(argc: *mut i32, argv: *mut *mut *mut c_char);
    fn hs_exit();
}

pub fn init() {
    HS_INIT.call_once(|| unsafe { hs_init(std::ptr::null_mut(), std::ptr::null_mut()) })
}

pub fn shutdown() {
    unsafe { hs_exit() }
}
