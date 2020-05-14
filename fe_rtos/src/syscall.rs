extern crate alloc;

use crate::fe_alloc;
use crate::task;
use crate::ipc;
use alloc::boxed::Box;
use alloc::vec::Vec;
use fe_osi::allocator::LayoutFFI;
use fe_osi::semaphore::Semaphore;
use fe_osi::ipc::Message;
use cstr_core::{CStr, c_char};

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
        unsafe {
            if (*sem).is_available() {
                break;
            } else {
                sys_yield();
            }
        }
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

#[no_mangle]
extern "C" fn sys_ipc_publish(c_topic: *const c_char, msg_ptr: *mut u8, msg_len: usize) -> usize {
    unsafe {
        match CStr::from_ptr(c_topic).to_str() {
            Ok(topic) => {
                let msg_vec = Vec::from_raw_parts(msg_ptr, msg_len, msg_len);
                ipc::TOPIC_REGISTERY_LOCK.take();
                ipc::TOPIC_REGISTERY.publish_to_topic(topic, &msg_vec);
                ipc::TOPIC_REGISTERY_LOCK.give();
                0
            },
            Err(_) => 1
        }
    }
}

#[no_mangle]
extern "C" fn sys_ipc_subscribe(c_topic: *const c_char) -> *const Semaphore {
    unsafe {
        let sem_ref = match CStr::from_ptr(c_topic).to_str() {
            Ok(topic) => {
                ipc::TOPIC_REGISTERY_LOCK.take();
                let sem = ipc::TOPIC_REGISTERY.subscribe_to_topic(topic);
                ipc::TOPIC_REGISTERY_LOCK.give();
                sem
            },
            Err(_) => { None }
        };
        match sem_ref {
            Some(s) => s,
            None => core::ptr::null()
        }
    }
}

#[no_mangle]
extern "C" fn sys_ipc_get_message(c_topic: *const c_char, sem: *const Semaphore) -> Message { 
    unsafe {
        let message = match CStr::from_ptr(c_topic).to_str() {
            Ok(topic) => {
                let sem_ref: &Semaphore = &*sem;
                sem_ref.take();

                ipc::TOPIC_REGISTERY_LOCK.take();
                let msg = ipc::TOPIC_REGISTERY.get_ipc_message(topic);
                ipc::TOPIC_REGISTERY_LOCK.give();
                msg
            },
            Err (_) => None,
        };
        match message {
            Some(valid_msg) => valid_msg.into(),
            None => Message { msg_ptr: core::ptr::null_mut(), msg_len: 0, valid: false }, 
        }
    }
}
