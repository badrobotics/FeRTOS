extern crate alloc;
use crate::r#yield;
use alloc::vec::Vec;
use cstr_core::{c_char, CString};

extern "C" {
    fn ipc_publish(topic: *const c_char, msg_ptr: *mut u8, msg_len: usize) -> usize;
    fn ipc_subscribe(topic: *const c_char) -> usize;
    fn ipc_get_message(topic: *const c_char) -> Message;
}

#[repr(C)]
pub struct Message {
    pub msg_ptr: *mut u8,
    pub msg_len: usize,
    pub valid: bool,
}

impl From<Vec<u8>> for Message {
    fn from(mut item: Vec<u8>) -> Self {
        item.shrink_to_fit();
        let (msg_ptr, msg_len, _msg_cap) = item.into_raw_parts();
        Message { msg_ptr, msg_len, valid: true}
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
    pub fn new(topic: &str) -> Self {
        let c_topic = CString::new(topic).unwrap();
        Publisher { topic: c_topic }
    }

    pub fn publish(&mut self, mut message: Vec<u8>) -> Result<(), &'static str>{
        message.shrink_to_fit();
        let (msg_ptr, msg_len, _msg_cap) = message.into_raw_parts();
        let success = unsafe {
            ipc_publish((&self.topic).as_ptr(), msg_ptr, msg_len)
        };
        if success == 0 {
            r#yield();
            Ok(())
        } else {
            Err("Unable to publish to topic.")
        }
    }
}

pub struct Subscriber {
    pub topic: CString,
}

impl Subscriber {
    pub fn new(topic: &str) -> Result<Self, &'static str> {
        let c_topic = CString::new(topic).unwrap();
        let resp = unsafe { ipc_subscribe((&c_topic).as_ptr()) };
        if resp == 1 {
            Err("Failed to subscribe to topic.")
        } else {
            Ok(Subscriber {
                topic: c_topic,
            })
        }
    }

    pub fn get_message(&mut self) -> Option<Vec<u8>> {
        let message: Message = unsafe {
            ipc_get_message((&self.topic).as_ptr())
        };
        if message.valid {
            Some(message.into())
        } else {
            None
        }
    }
}
