extern crate alloc;
use crate::r#yield;
use alloc::vec::Vec;
use cstr_core::{c_char, CString};

extern "C" {
    fn ipc_publish(topic: *const c_char, msg_ptr: *mut u8, msg_len: usize) -> usize;
    fn ipc_subscribe(topic: *const c_char) -> usize;
    fn ipc_unsubscribe(topic: *const c_char) -> usize;
    fn ipc_get_message(topic: *const c_char, block: bool) -> Message;
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
        Message {
            msg_ptr,
            msg_len,
            valid: true,
        }
    }
}

impl From<Message> for Vec<u8> {
    fn from(msg: Message) -> Vec<u8> {
        unsafe { Vec::from_raw_parts(msg.msg_ptr, msg.msg_len, msg.msg_len) }
    }
}

/// A data structure that allows tasks to publish messages to an IPC topic
pub struct Publisher {
    pub topic: CString,
}

impl Publisher {
    /// Creates a new Publisher for the specified topic
    ///
    /// #Examples
    ///
    /// ```
    /// let my_publisher = Publisher::new("my topic").unwrap();
    /// ```
    pub fn new(topic: &str) -> Result<Self, &'static str> {
        let c_topic = match CString::new(topic) {
            Ok(t) => t,
            Err(_) => return Err("Invalid topic string"),
        };
        Ok(Publisher { topic: c_topic })
    }

    /// Publishes a message to the topic.
    ///
    /// #Examples
    /// ```
    /// my_publisher.publish("Hello, World!".into_bytes());
    /// ```
    pub fn publish(&mut self, mut message: Vec<u8>) -> Result<(), &'static str> {
        message.shrink_to_fit();
        let (msg_ptr, msg_len, _msg_cap) = message.into_raw_parts();
        let success = unsafe { ipc_publish((&self.topic).as_ptr(), msg_ptr, msg_len) };
        if success == 0 {
            r#yield();
            Ok(())
        } else {
            Err("Unable to publish to topic.")
        }
    }
}

/// A data structure that allows tasks to subscribe to a topic
pub struct Subscriber {
    pub topic: CString,
}

impl Subscriber {
    /// Creates a new Subscriber for the specified topic
    ///
    /// ```
    /// let my_subscriber = Subscriber::new("my topic").unwrap();
    /// ```
    pub fn new(topic: &str) -> Result<Self, &'static str> {
        let c_topic = match CString::new(topic) {
            Ok(t) => t,
            Err(_) => return Err("Invalid topic string"),
        };
        let new_sub = Subscriber { topic: c_topic };
        let resp = unsafe { ipc_subscribe((&new_sub.topic).as_ptr()) };
        if resp == 1 {
            Err("Failed to subscribe to topic.")
        } else {
            Ok(new_sub)
        }
    }

    fn get_message_base(&mut self, block: bool) -> Option<Vec<u8>> {
        let message: Message = unsafe { ipc_get_message((&self.topic).as_ptr(), block) };
        if message.valid {
            Some(message.into())
        } else {
            None
        }
    }

    /// Returns the next message from the topic.
    /// Blocks if there are no new messages.
    pub fn get_message(&mut self) -> Option<Vec<u8>> {
        self.get_message_base(true)
    }

    /// Returns the next message from the topic if there is one.
    /// If there are no new messages, it returns None
    pub fn get_message_nonblocking(&mut self) -> Option<Vec<u8>> {
        self.get_message_base(false)
    }
}
impl Drop for Subscriber {
    fn drop(&mut self) {
        unsafe {
            ipc_unsubscribe((&self.topic).as_ptr());
        }
    }
}
