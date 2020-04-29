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
    pub(crate) fn new(name: &String) -> Topic {
        Topic {
            name: name.clone(),
            data: Vec::new(),
            subscribers: BTreeMap::new(),
        }
    }

    pub(crate) fn add_message(&mut self, message: &Vec<u8>) {
        self.data.push(message.clone());
        for (_pid, subscriber) in &mut self.subscribers {
            subscriber.set_available();
        }
    }

    pub(crate) fn add_subscriber(&mut self, pid: usize, mut subscriber: Subscriber) {
        // set the index of the new subscriber to the end of the queue
        subscriber.index = self.data.len();

        // take the condition variable to indicate no new messages
        subscriber.set_unavailable();

        self.subscribers.insert(pid, subscriber);
    }
}