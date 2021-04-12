use core::ffi::c_void;

/// trampoline function for preserve closure type information
pub unsafe extern "C" fn trampoline<F>(user_data: *mut c_void, ptr: *const u8, size: usize)
    where
        F: FnMut(&[u8]),
{
    (*(user_data as *mut F))(alloc::slice::from_raw_parts(ptr, size))
}