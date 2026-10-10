//! C ABI。C# などの他言語から P/Invoke で呼ぶための入口。
//!
//! 文字列は UTF-8 の C 文字列で受け渡し、戻り値の JSON は呼び出し側で
//! `unitconv_free` を呼んで解放する。

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::panic;

fn read(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned()
}

fn into_c(json: String) -> *mut c_char {
    CString::new(json).unwrap_or_else(|_| CString::new("{}").unwrap()).into_raw()
}

fn error_json(message: &str) -> String {
    format!(
        "{{\"ok\":false,\"value\":null,\"unit\":null,\"pretty\":null,\"error\":\"{message}\"}}"
    )
}

fn guard<F: FnOnce() -> String + panic::UnwindSafe>(body: F) -> *mut c_char {
    match panic::catch_unwind(body) {
        Ok(json) => into_c(json),
        Err(_) => into_c(error_json("内部エラー（Rust 側でパニック）")),
    }
}

/// `unitconv <value> <from> <to>` 相当。JSON を返す。
#[no_mangle]
pub extern "C" fn unitconv_convert(
    value: *const c_char,
    from: *const c_char,
    to: *const c_char,
) -> *mut c_char {
    guard(|| crate::convert(&read(value), &read(from), &read(to)).to_json())
}

/// 接頭辞スケーリング。JSON を返す。
#[no_mangle]
pub extern "C" fn unitconv_scale(
    value: *const c_char,
    from: *const c_char,
    to: *const c_char,
) -> *mut c_char {
    guard(|| crate::scale(&read(value), &read(from), &read(to)).to_json())
}

/// 接頭辞一覧。JSON 配列を返す。
#[no_mangle]
pub extern "C" fn unitconv_prefixes() -> *mut c_char {
    guard(crate::prefixes_json)
}

/// 単位一覧。JSON 配列を返す。
#[no_mangle]
pub extern "C" fn unitconv_units() -> *mut c_char {
    guard(crate::units_json)
}

/// `unitconv_*` が返した文字列を解放する。
#[no_mangle]
pub extern "C" fn unitconv_free(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(CString::from_raw(ptr));
    }
}

/// wasm 用: JS が入力文字列を書き込むためのバッファを確保する（NUL 終端ぶん込み）。
#[no_mangle]
pub extern "C" fn unitconv_alloc(len: usize) -> *mut u8 {
    let mut buffer = Vec::<u8>::with_capacity(len + 1);
    let pointer = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    pointer
}

/// `unitconv_alloc` で確保したバッファを解放する。
#[no_mangle]
pub extern "C" fn unitconv_dealloc(pointer: *mut u8, len: usize) {
    if pointer.is_null() {
        return;
    }
    unsafe {
        let _ = Vec::from_raw_parts(pointer, 0, len + 1);
    }
}
