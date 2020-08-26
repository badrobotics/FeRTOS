extern crate alloc;
use crate::semaphore::Semaphore;
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::ptr::null_mut;

extern "C" {
    fn do_task_spawn(
        stack_size: usize,
        entry_point: *const u32,
        parameter: *mut u32,
    ) -> *const Semaphore;
}

/// Creates a new task that will be run.
///
/// #Examples
/// ```
/// fn test_task(_: &mut u8) {
///     loop {}
/// }
/// task_spawn(1024, test_task, None);
/// ```
pub fn task_spawn<T: Send>(
    stack_size: usize,
    entry_point: fn(&mut T),
    parameter: Option<Box<T>>,
) -> Arc<Semaphore> {
    unsafe {
        let param_ptr = match parameter {
            Some(param) => Box::into_raw(param) as *mut u32,
            None => null_mut(),
        };
        let raw_sem = do_task_spawn(stack_size, entry_point as *const u32, param_ptr);
        Arc::from_raw(raw_sem)
    }
}

/// Creates a new task that will be run and blocks until that task exits.
pub fn task_spawn_block<T: Send>(
    stack_size: usize,
    entry_point: fn(&mut T),
    parameter: Option<Box<T>>,
) {
    let sem = task_spawn(stack_size, entry_point, parameter);
    sem.take();
}
