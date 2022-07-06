#![allow(non_snake_case)]

use core::ffi::c_void;

// TODO don't have these depend on the libc crate; instead, use default
// allocator, built-in memset, etc.

#[no_mangle]
pub unsafe extern "C" fn roc_alloc(size: usize, _alignment: u32) -> *mut c_void {
    return libc::malloc(size);
}

#[no_mangle]
pub unsafe extern "C" fn roc_realloc(
    c_ptr: *mut c_void,
    new_size: usize,
    _old_size: usize,
    _alignment: u32,
) -> *mut c_void {
    return libc::realloc(c_ptr, new_size);
}

#[no_mangle]
pub unsafe extern "C" fn roc_dealloc(c_ptr: *mut c_void, _alignment: u32) {
    return libc::free(c_ptr);
}

#[no_mangle]
pub unsafe extern "C" fn roc_panic(c_ptr: *mut c_void, tag_id: u32) {
    use std::ffi::CStr;
    use std::os::raw::c_char;

    match tag_id {
        0 => {
            let slice = CStr::from_ptr(c_ptr as *const c_char);
            let string = slice.to_str().unwrap();
            eprintln!("Roc hit a panic: {}", string);
            std::process::exit(1);
        }
        _ => todo!(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn roc_memcpy(dst: *mut c_void, src: *mut c_void, n: usize) -> *mut c_void {
    libc::memcpy(dst, src, n)
}

#[no_mangle]
pub unsafe extern "C" fn roc_memset(dst: *mut c_void, c: i32, n: usize) -> *mut c_void {
    libc::memset(dst, c, n)
}

////////////////////////////////////////////////////////////////////////////
//
// TODO: rust_main should be removed once we use surgical linking everywhere.
// It's just a workaround to get cargo to build an object file the way
// the non-surgical linker needs it to. The surgical linker works on
// executables, not object files, so this workaround is not needed there.
//
////////////////////////////////////////////////////////////////////////////
#[no_mangle]
pub extern "C" fn rust_main() -> i32 {
    use roc_std::RocStr;

    unsafe {
        let roc_str = roc_main();

        let len = roc_str.len();
        let str_bytes = roc_str.as_bytes().as_ptr() as *const libc::c_void;

        if libc::write(1, str_bytes, len) < 0 {
            panic!("Writing to stdout failed!");
        }
    }

    // Exit code
    0
}
