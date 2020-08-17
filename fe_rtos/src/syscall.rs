extern crate alloc;

use crate::arch;
use crate::fe_alloc;
use crate::ipc;
use crate::task;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use cstr_core::{c_char, CStr};
use fe_osi::allocator::LayoutFFI;
use fe_osi::ipc::Message;
use fe_osi::semaphore::Semaphore;

//For the linker to link the syscalls, a function in this
//file must be called from elsewhere...
pub fn link_syscalls() {}

#[no_mangle]
extern "C" fn sys_exit() -> usize {
    unsafe {
        ipc::TOPIC_REGISTERY_LOCK.with_lock(|| {
            let topics: Vec<String> = ipc::TOPIC_REGISTERY.topic_lookup.keys().cloned().collect();
            for topic in topics {
                ipc::TOPIC_REGISTERY.unsubscribe_from_topic(&topic);
            }
        });

        task::get_cur_task().give();

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
    unsafe { fe_alloc::alloc(layout) }
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
) -> *const Semaphore {
    let sem = Arc::new(Semaphore::new(0));
    let task_info = Box::new(task::NewTaskInfo {
        ep: entry_point,
        param: parameter,
        sem: Arc::clone(&sem),
    });

    unsafe {
        task::add_task(
            stack_size,
            task::new_task_helper as *const usize,
            Box::into_raw(task_info) as *mut usize,
        );
    }

    Arc::into_raw(sem)
}

#[no_mangle]
extern "C" fn sys_yield() -> usize {
    unsafe {
        task::do_context_switch();
    }

    0
}

#[no_mangle]
extern "C" fn sys_ipc_publish(c_topic: *const c_char, msg_ptr: *mut u8, msg_len: usize) -> usize {
    unsafe {
        let msg_vec = Vec::from_raw_parts(msg_ptr, msg_len, msg_len);

        let topic: &str = match CStr::from_ptr(c_topic).to_str() {
            Ok(topic) => topic,
            Err(_) => {
                // return early indicating failure
                return 1;
            }
        };

        ipc::TOPIC_REGISTERY_LOCK.with_lock(|| {
            ipc::TOPIC_REGISTERY.publish_to_topic(topic, &msg_vec);
        });
        0
    }
}

#[no_mangle]
extern "C" fn sys_ipc_subscribe(c_topic: *const c_char) -> usize {
    unsafe {
        let topic: &str = match CStr::from_ptr(c_topic).to_str() {
            Ok(topic) => topic,
            Err(_) => {
                // return early indicating failure
                return 1;
            }
        };

        ipc::TOPIC_REGISTERY_LOCK.with_lock(|| {
            ipc::TOPIC_REGISTERY.subscribe_to_topic(topic);
        });
        0
    }
}

#[no_mangle]
extern "C" fn sys_ipc_unsubscribe(c_topic: *const c_char) -> usize {
    unsafe {
        let topic: &str = match CStr::from_ptr(c_topic).to_str() {
            Ok(topic) => topic,
            Err(_) => {
                // return early indicating failure
                return 1;
            }
        };

        let mut success = 1;
        ipc::TOPIC_REGISTERY_LOCK.with_lock(|| {
            success = match ipc::TOPIC_REGISTERY.unsubscribe_from_topic(topic) {
                Some(_) => 0,
                None => 1,
            }
        });
        success
    }
}

fn get_null_message() -> Message {
    Message {
        msg_ptr: core::ptr::null_mut(),
        msg_len: 0,
        valid: false,
    }
}

#[no_mangle]
extern "C" fn sys_ipc_get_message(c_topic: *const c_char, block: bool) -> Message {
    let mut sem_ref: Option<&Semaphore> = None;
    unsafe {
        let topic = match CStr::from_ptr(c_topic).to_str() {
            Ok(t) => t,
            Err(_) => return get_null_message(),
        };

        ipc::TOPIC_REGISTERY_LOCK.with_lock(|| {
            sem_ref = ipc::TOPIC_REGISTERY.get_subscriber_lock(topic);
        });

        if sem_ref.is_none() {
            return get_null_message();
        }

        if block {
            sem_ref.unwrap().take();
        } else if !sem_ref.unwrap().try_take() {
            return get_null_message();
        }

        let mut message: Option<Vec<u8>> = None;
        ipc::TOPIC_REGISTERY_LOCK.with_lock(|| {
            message = ipc::TOPIC_REGISTERY.get_ipc_message(topic);
        });

        match message {
            Some(msg) => msg.into(),
            None => get_null_message(),
        }
    }
}

#[no_mangle]
extern "C" fn sys_get_heap_remaining() -> usize {
    fe_alloc::get_heap_remaining()
}

#[no_mangle]
extern "C" fn sys_interrupt_register(irqn: isize, handler: *const usize) -> usize {
    unsafe {
        arch::int_register(irqn, handler);
    }
    0
}
