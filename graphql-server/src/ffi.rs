use std::ffi::{CStr, CString};

use libc::c_char;

unsafe extern "C" {

    pub fn abs_parse_sdmx(json: *const c_char) -> *mut c_char;
    pub fn abs_free_string(ptr: *mut c_char) -> ();

}

pub fn parse_sdmx(json: &str) -> Result<String, String> {
    crate::hs_runtime::init();

    let input = CString::new(json).map_err(|e| e.to_string())?;

    let result_ptr = unsafe { abs_parse_sdmx(input.as_ptr()) };

    if result_ptr.is_null() {
        return Err("Haskell returned null".into());
    }

    let result = unsafe { CStr::from_ptr(result_ptr) };

    Ok(result.to_str().map_err(|e| e.to_string())?.to_string())
}
