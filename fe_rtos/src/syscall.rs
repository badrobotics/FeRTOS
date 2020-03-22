extern crate alloc;

use crate::fe_alloc;
use crate::ipc::IPCMessage;
use crate::task;
use crate::task::NEW_MESSAGE_QUEUE;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use cstr_core::CString;
use fe_osi::allocator::LayoutFFI;
use fe_osi::ipc::Message;
use fe_osi::semaphore::Semaphore;

extern "C" {
    pub fn svc_handler();
}

//For the linker to link the syscalls, a function in this
//file must be called from elsewhere...
pub fn link_syscalls() {}

#[no_mangle]
extern "C" fn sys_exit() -> usize {
    unsafe {
        while !task::remove_task() {
            sys_yield();
        }

        0
    }
}

#[no_mangle]
extern "C" fn sys_sleep(ms32: u32) -> usize {
    let ms: u64 = ms32 as u64;

    while !task::sleep(ms) {
        sys_yield();
    }

    0
}

#[no_mangle]
extern "C" fn sys_alloc(layout: LayoutFFI) -> *mut u8 {
    unsafe {
        return fe_alloc::alloc(layout);
    }
}

#[no_mangle]
extern "C" fn sys_dealloc(ptr: *mut u8, layout: LayoutFFI) -> usize {
    unsafe {
        fe_alloc::dealloc(ptr, layout);
    }

    0
}

#[no_mangle]
extern "C" fn sys_block(sem: *const Semaphore) -> usize {
    while !task::block(sem) {
        sys_yield();
    }

    0
}

#[no_mangle]
extern "C" fn sys_task_spawn(
    stack_size: usize,
    entry_point: *const u32,
    parameter: *mut u32,
) -> usize {
    let task_info = Box::new(task::NewTaskInfo {
        ep: entry_point,
        param: parameter,
    });

    unsafe {
        task::add_task(
            stack_size,
            task::new_task_helper as *const u32,
            Box::into_raw(task_info) as *mut u32,
        );
    }

    0
}

#[no_mangle]
extern "C" fn sys_yield() -> usize {
    unsafe { task::do_context_switch(); }

    0
}

extern "C" fn sys_ipc_publish(c_topic: *const u8, message: Message) {
    let topic: String;
    unsafe {
        topic = CString::from_raw(c_topic as *mut u8).into_string().unwrap();
    }
    let msg = IPCMessage {
        topic: topic,
        data: message.into(),
    };
    NEW_MESSAGE_QUEUE.push(msg);
}

#[no_mangle]
extern "C" fn sys_ipc_subscribe(topic: *const u8) {}

//#[no_mangle]
//extern "C" fn sys_ipc_get_message(topic: *const u8) -> Message { }
