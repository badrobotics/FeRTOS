extern crate alloc;
use alloc::boxed::Box;

extern "C" {
    fn do_task_spawn(stack_size: usize, entry_point: *const u32, parameter: *mut u32);  
}

pub fn task_spawn<T: Send>(stack_size: usize, entry_point: fn(&mut T), parameter: Option<Box<T>>) {
    unsafe {
        let param_ptr = match parameter {
            Some(param) => Box::into_raw(param) as *mut u32,
            None => 0 as *mut u32
        };
        do_task_spawn(stack_size, entry_point as *const u32, param_ptr);
    }
}
