extern crate alloc;
use alloc::collections::BTreeMap;
use core::cell::RefCell;
use crate::ipc::subscriber::Subscriber;
use crate::ipc::subscriber::MessageNode;
use alloc::sync::Arc;
use alloc::string::String;

pub(crate) struct Topic {
    pub(crate) name: String,
    pub(crate) tail: Option<Arc<MessageNode>>,
    pub(crate) subscribers: BTreeMap<usize, Subscriber>,
}

impl Topic {
    pub(crate) fn new(name: &str) -> Topic {
        Topic {
            name: String::from(name),
            tail: None,
            subscribers: BTreeMap::new(),
        }
    }

    pub(crate) fn add_message(&mut self, message: &[u8]) {
        let new_tail = Arc::new(MessageNode {
            data: message.to_vec(),
            next: RefCell::new(None),
        });

        // Add the new message to the end of the list
        match &self.tail {
            Some(m) => {
                m.next.replace(Some(Arc::clone(&new_tail)));
            },
            None => (),
        };
        // Point tail to the end of the list
        self.tail = Some(Arc::clone(&new_tail));

        for subscriber in &mut self.subscribers.values_mut() {
            match subscriber.next_message {
                Some(_) => (),
                None => subscriber.next_message = Some(Arc::clone(&new_tail)),
            }
            subscriber.lock.give();
        }
    }

    pub(crate) fn add_subscriber(&mut self, pid: usize, mut subscriber: Subscriber) {
        subscriber.next_message = None;
        self.subscribers.insert(pid, subscriber);
    }
}
