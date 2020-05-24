extern "C" {
    fn do_register_interrupt(irqn: isize, entry_point: *const usize) -> usize;
}

pub fn register_interrupt(irqn: isize, entry_point: unsafe extern "C" fn()) {
    unsafe {
        do_register_interrupt(irqn, entry_point as *const usize);
    }
}
