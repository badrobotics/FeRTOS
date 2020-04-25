extern crate alloc;
use crate::semaphore::Semaphore;
use alloc::string::String;
use alloc::vec::Vec;
use cstr_core::CString;

extern "C" {
    fn ipc_publish(topic: *const u8, message: Message);
    fn ipc_subscribe(topic: *const u8, sem: *const Semaphore);
    fn ipc_get_message(topic: *const u8, sem: *const Semaphore) -> Message;
}

#[repr(C)]
pub struct Message {
    msg_ptr: *mut u8,
    msg_len: usize,
}

impl From<Vec<u8>> for Message {
    fn from(mut item: Vec<u8>) -> Self {
        item.shrink_to_fit();
        let (msg_ptr, msg_len, _msg_cap) = item.into_raw_parts();
        Message {
            msg_ptr: msg_ptr,
            msg_len: msg_len,
        }
    }
}

impl Into<Vec<u8>> for Message {
    fn into(self) -> Vec<u8> {
        unsafe { Vec::from_raw_parts(self.msg_ptr, self.msg_len, self.msg_len) }
    }
}

pub struct Publisher {
    pub topic: CString,
}

impl Publisher {
    pub fn new(topic: String) -> Self {
        let c_topic = CString::new(topic).unwrap();
        Publisher { topic: c_topic }
    }

    pub fn publish(&mut self, message: Vec<u8>) {
        let c_msg: Message = message.into();
        unsafe {
            ipc_publish(self.topic.as_ptr(), c_msg);
        }
    }
}

pub struct Subscriber {
    pub topic: CString,
    sem: Semaphore,
}

impl<'a> Subscriber {
    pub fn new(&mut self, topic: String) -> Self {
        let c_topic: CString = CString::new(topic).unwrap();
        let subscriber = Subscriber {
            topic: c_topic,
            sem: Semaphore::new_mutex(),
        };
        unsafe {
            ipc_subscribe(self.topic.as_ptr(), &self.sem);
        }
        subscriber
    }

    pub fn get_message(&mut self) -> Option<Vec<u8>> {
        if self.sem.is_available() {
            let message: Message;
            unsafe {
                message = ipc_get_message(self.topic.as_ptr(), &self.sem);
            };
            Some(message.into())
        } else {
            None
        }
    }
}
