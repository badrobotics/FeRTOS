extern crate alloc;
extern crate crossbeam_queue;
use crossbeam_queue::SegQueue;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;

pub (crate) struct IPCMessage {
    pub (crate) topic: String,
    pub (crate) data: Vec<u8>, 
}

pub(crate) struct TopicRegistery {
    map: BTreeMap<String, SegQueue<Vec<u8>>>,
}

impl TopicRegistery {
    pub fn new() -> Self {
        TopicRegistery {
            map: BTreeMap::new()
        }
    }

    pub fn register_topic(&mut self, topic: String) {
        match self.map.get(&topic) {
            None => { 
                self.map.insert(topic, SegQueue::new());
            }
            Some(_) => { }
        }
    }

    pub fn publish_to_topic(&mut self, topic: String, message: Vec<u8>) {
        self.map.entry(topic)
            .or_insert_with(SegQueue::new)
            .push(message);
    }
}