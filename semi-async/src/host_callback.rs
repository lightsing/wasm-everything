use alloc::vec::Vec;
use core::ffi::c_void;

/// trampoline function for preserve closure type information
/// `ptr`, `size` and `cap` comes from a Vec<u8>
pub unsafe fn trampoline<F>(user_data: *mut c_void, ptr: *mut u8, size: usize, cap: usize) where F: FnMut(Vec<u8>) {
    let data = unsafe { Vec::from_raw_parts(ptr, size, cap) };
    (*(user_data as *mut F))(data)
}