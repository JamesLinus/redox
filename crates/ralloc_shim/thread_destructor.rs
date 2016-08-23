/// Does this platform support thread destructors?
///
/// This will always return true.
#[inline]
pub fn is_supported() -> bool { true }

/// Register a thread destructor.
///
/// # Safety
///
/// This is unsafe due to accepting (and dereferencing) raw pointers, as well as running an
/// arbitrary unsafe function.
pub fn register(_t: *mut u8, _dtor: unsafe extern fn(*mut u8)) {}
