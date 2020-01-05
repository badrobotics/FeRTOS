use crate::task;
use crate::fe_alloc;
use fe_osi::allocator::LayoutFFI;
use fe_osi::semaphore::Semaphore;

extern "C" {
    pub fn svc_handler();
}

//For the linker to link the syscalls, a function in this
//file must be called from elsewhere...
pub fn link_syscalls(){}

#[no_mangle]
extern "C" fn sys_exit() -> usize {
    unsafe { task::remove_task(); }

    0
}

#[no_mangle]
extern "C" fn sys_sleep(ms32: u32) -> usize {
    let ms: u64 = ms32 as u64;
    task::sleep(ms);

    0
}

#[no_mangle]
extern "C" fn sys_alloc(layout: LayoutFFI) -> *mut u8 {
    unsafe { return fe_alloc::alloc(layout); }
}

#[no_mangle]
extern "C" fn sys_dealloc(ptr: *mut u8, layout: LayoutFFI)  -> usize {
    unsafe { fe_alloc::dealloc(ptr, layout); }

    0
}

#[no_mangle]
extern "C" fn sys_block(sem: *const Semaphore) -> usize {
    task::block(sem);

    0
}

#[no_mangle]
extern "C" fn sys_task_spawn(stack_size: usize, entry_point: *const u32, parameter: *mut u32) -> usize {
    unsafe { task::add_task(stack_size, entry_point, parameter); }

    0
}
