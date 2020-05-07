extern crate alloc;

use crate::fe_alloc;
use crate::task;
use crate::ipc;
use alloc::boxed::Box;
use alloc::vec::Vec;
use fe_osi::allocator::LayoutFFI;
use fe_osi::semaphore::Semaphore;
use fe_osi::ipc::Message;

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
extern "C" fn sys_ipc_publish(topic_ptr: *mut u8, topic_len: usize, msg_ptr: *mut u8, msg_len: usize) -> usize{
    unsafe {
        let msg: Vec<u8> = Vec::from_raw_parts(msg_ptr, msg_len, msg_len);
        let topic: Vec<u8> = Vec::from_raw_parts(topic_ptr, topic_len, topic_len);
        ipc::TOPIC_REGISTERY_LOCK.take();
        ipc::TOPIC_REGISTERY.publish_to_topic(&topic, msg);
        ipc::TOPIC_REGISTERY_LOCK.give();
        Box::into_raw(Box::new(topic));
    }

    0
}

#[no_mangle]
extern "C" fn sys_ipc_subscribe(topic_ptr: *mut u8, topic_len: usize) -> *const Semaphore {
    let sem: Semaphore = Semaphore::new(0);
    unsafe {
        let topic: Vec<u8> = Vec::from_raw_parts(topic_ptr, topic_len, topic_len);
        ipc::TOPIC_REGISTERY_LOCK.take();
        let sem_ref = ipc::TOPIC_REGISTERY.subscribe_to_topic(&topic, sem);
        ipc::TOPIC_REGISTERY_LOCK.give();
        Box::into_raw(Box::new(topic));
        sem_ref
    }
}

#[no_mangle]
extern "C" fn sys_ipc_get_message(topic_ptr: *mut u8, topic_len: usize, sem: *const Semaphore) -> Message { 
    unsafe {
        let sem_ref: &Semaphore = &*sem;
        let topic: Vec<u8> = Vec::from_raw_parts(topic_ptr, topic_len, topic_len);

        sem_ref.take();
        ipc::TOPIC_REGISTERY_LOCK.take();
        let msg = ipc::TOPIC_REGISTERY.get_ipc_message(&topic).unwrap();
        ipc::TOPIC_REGISTERY_LOCK.give();
        Box::into_raw(Box::new(topic));

        msg.clone().into()
    }
}
