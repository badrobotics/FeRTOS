extern crate alloc;
use alloc::collections::BTreeMap;

use crate::ipc::subscriber::Subscriber;
use alloc::vec::Vec;

pub(crate) struct Topic {
    pub(crate) name: Vec<u8>,
    pub(crate) data: Vec<Vec<u8>>,
    pub(crate) subscribers: BTreeMap<usize, Subscriber>,
}

impl Topic {
    pub(crate) fn new(name: &Vec<u8>) -> Topic {
        Topic {
            name: name.clone(),
            data: Vec::new(),
            subscribers: BTreeMap::new(),
        }
    }

    pub(crate) fn add_message(&mut self, message: Vec<u8>) {
        self.data.push(message);
        for (_pid, subscriber) in &mut self.subscribers {
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
        for (_pid, subscriber) in &mut self.subscribers {
            indicies.push(subscriber.index);
        }
        let min_index = match indicies.iter().min() {
            Some(min) => *min,
            None => 0,
        };
        self.data.drain(0..(min_index));
        for (_pid, subscriber) in &mut self.subscribers {
            subscriber.index = subscriber.index - (min_index);
        }
        min_index
    }
}
