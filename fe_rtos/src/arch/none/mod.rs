pub(crate) unsafe fn set_canary(_stack_bottom: *const usize, _stack_size: usize) -> *const usize {
    core::ptr::null()
}

pub(crate) unsafe fn set_initial_stack(
    _stack_bottom: *const usize,
    _stack_size: usize,
    _entry_point: *const usize,
    _param: *mut usize,
) -> *const usize {
    core::ptr::null()
}

pub fn int_register(_irqn: i8, _int_handler: *const usize) {}
