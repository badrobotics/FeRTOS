extern crate alloc;
use alloc::collections::BTreeMap;

use crate::ipc::subscriber::Subscriber;
use alloc::vec::Vec;
use alloc::string::String;

pub(crate) struct Topic {
    pub(crate) name: String,
    pub(crate) data: Vec<Vec<u8>>,
    pub(crate) subscribers: BTreeMap<usize, Subscriber>,
}

impl Topic {
    pub(crate) fn new(name: &str) -> Topic {
        Topic {
            name: String::from(name),
            data: Vec::new(),
            subscribers: BTreeMap::new(),
        }
    }

    pub(crate) fn add_message(&mut self, message: &[u8]) {
        self.data.push(message.to_vec());
        for subscriber in &mut self.subscribers.values_mut() {
            subscriber.lock.give();
        }
    }

    pub(crate) fn add_subscriber(&mut self, pid: usize, mut subscriber: Subscriber) {
        // set the index of the new subscriber to the end of the queue
        subscriber.index = self.data.len();
        self.subscribers.insert(pid, subscriber);
    }

    pub(crate) fn cleanup(&mut self) -> usize {
        let mut indicies: Vec<usize> = Vec::new();
        for subscriber in &mut self.subscribers.values_mut() {
            indicies.push(subscriber.index);
        }
        let min_index = match indicies.iter().min() {
            Some(min) => *min,
            None => 0,
        };
        self.data.drain(0..(min_index));
        for subscriber in &mut self.subscribers.values_mut() {
            subscriber.index -= min_index;
        }
        min_index
    }
}
