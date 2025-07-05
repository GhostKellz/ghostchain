use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use anyhow::Result;
use crate::types::*;

pub mod zquic;
pub mod zquic_integration;
pub mod ghostbridge;
pub mod ghostbridge_integration;

/// C-compatible result type for FFI
#[repr(C)]
pub struct FFIResult {
    pub success: bool,
    pub error_code: c_int,
    pub error_message: *const c_char,
    pub data: *mut c_void,
}

impl FFIResult {
    pub fn success(data: *mut c_void) -> Self {
        Self {
            success: true,
            error_code: 0,
            error_message: ptr::null(),
            data,
        }
    }

    pub fn error(code: c_int, message: &str) -> Self {
        let c_message = CString::new(message).unwrap_or_default();
        Self {
            success: false,
            error_code: code,
            error_message: c_message.into_raw(),
            data: ptr::null_mut(),
        }
    }
}

/// Convert Rust string to C string
pub fn rust_to_c_string(s: &str) -> *mut c_char {
    CString::new(s).unwrap_or_default().into_raw()
}

/// Convert C string to Rust string
pub unsafe fn c_to_rust_string(ptr: *const c_char) -> Result<String> {
    if ptr.is_null() {
        return Ok(String::new());
    }
    unsafe {
        Ok(CStr::from_ptr(ptr).to_string_lossy().into_owned())
    }
}

/// Free C string allocated by Rust
#[unsafe(no_mangle)]
pub extern "C" fn ghostchain_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe { CString::from_raw(ptr) };
    }
}

/// Free FFI result
#[unsafe(no_mangle)]
pub extern "C" fn ghostchain_free_result(result: *mut FFIResult) {
    if !result.is_null() {
        unsafe {
            if !(*result).error_message.is_null() {
                CString::from_raw((*result).error_message as *mut c_char);
            }
            Box::from_raw(result);
        }
    }
}