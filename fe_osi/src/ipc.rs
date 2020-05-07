extern crate alloc;
use crate::semaphore::Semaphore;
use alloc::vec::Vec;

extern "C" {
    fn ipc_publish(topic_ptr: *const u8, topic_len: usize, message_ptr: *mut u8, msg_len: usize);
    fn ipc_subscribe(topic_ptr: *const u8, topic_len: usize) -> *const Semaphore;
    fn ipc_get_message(topic_ptr: *const u8, topic_len: usize, sem: *const Semaphore) -> Message;
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
    pub topic: Vec<u8>,
}

impl Publisher {
    pub fn new(topic: &str) -> Self {
        Publisher { topic: topic.as_bytes().to_vec() }
    }

    pub fn publish(&mut self, message: Vec<u8>) {
        unsafe {
            let (msg_ptr, msg_len, _msg_cap) = message.into_raw_parts();
            ipc_publish(
                self.topic.as_ptr(),
                self.topic.len(),
                msg_ptr,
                msg_len,
            );
        }
    }
}

pub struct Subscriber {
    pub topic: Vec<u8>,
    sem: *const Semaphore,
}

impl Subscriber {
    pub fn new(topic: &str) -> Self {
        let vec_topic = topic.as_bytes().to_vec();

        let sem = unsafe {
            ipc_subscribe(
                vec_topic.as_ptr(),
                vec_topic.len(),
            )
        };

        let subscriber = Subscriber {
            topic: vec_topic,
            sem: sem,
        };
        subscriber
    }

    pub fn get_message(&mut self) -> Vec<u8> {
        let message: Message;
        unsafe {
            message = ipc_get_message(
                self.topic.as_ptr(),
                self.topic.len(),
                self.sem
            );
        };
        message.into()
    }
}
