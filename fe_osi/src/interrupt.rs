extern "C" {
    fn do_register_interrupt(irqn: isize, entry_point: *const usize) -> usize;
}

/// Registers a function to be the handler for the specified interrupt.
///
/// # Examples
/// ```
/// register_interrupt(I2C1_IRQN, i2c_handler);
/// ```
pub fn register_interrupt(irqn: isize, entry_point: unsafe extern "C" fn()) {
    unsafe {
        do_register_interrupt(irqn, entry_point as *const usize);
    }
}
