extern crate alloc;
extern crate crossbeam_queue;
use crossbeam_queue::SegQueue;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::string::String;


struct TopicRegistery {
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
}