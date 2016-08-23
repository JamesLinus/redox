use system::syscall::unix::{sys_brk, sys_yield};

/// Change the data segment. See `man brk`.
///
/// On success, the new program break is returned. On failure, the old program break is returned.
///
/// # Note
///
/// This is the `brk` **syscall**, not the library function.
pub unsafe fn brk(ptr: *const u8) -> *const u8 {
    sys_brk(ptr as usize).unwrap_or(0) as *const u8
}

/// Voluntarily give a time slice to the scheduler.
pub fn sched_yield() -> usize {
    sys_yield().unwrap_or(!0)
}
