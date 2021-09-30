mod subscriber;
mod topic;

extern crate alloc;
use crate::ipc::subscriber::{MessageNode, Subscriber};
use crate::ipc::topic::Topic;
use crate::task::get_cur_task;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use fe_osi::semaphore::Semaphore;

pub(crate) struct TopicRegistry {
    pub(crate) topic_lookup: BTreeMap<String, Topic>,
}

pub(crate) static mut TOPIC_REGISTERY_LOCK: Semaphore = Semaphore::new_mutex();
pub(crate) static mut TOPIC_REGISTERY: TopicRegistry = TopicRegistry {
    topic_lookup: BTreeMap::new(),
};

impl TopicRegistry {
    pub(crate) fn publish_to_topic(&mut self, message_topic: &str, message: &[u8]) {
        let owned_topic = String::from(message_topic);
        self.topic_lookup
            .entry(owned_topic)
            .and_modify(|topic| topic.add_message(message));
    }

    pub(crate) fn subscribe_to_topic(&mut self, subscriber_topic: &str) {
        let pid: usize = unsafe { get_cur_task().pid };
        let subscriber = Subscriber::new();
        let owned_topic = String::from(subscriber_topic);
        self.topic_lookup
            .entry(owned_topic)
            .or_insert_with(Topic::new)
            .add_subscriber(pid, subscriber);
    }

    pub(crate) fn unsubscribe_from_topic(&mut self, subscriber_topic: &str) -> Option<Subscriber> {
        let pid: usize = unsafe { get_cur_task().pid };
        self.topic_lookup
            .get_mut(subscriber_topic)
            .and_then(|topic| topic.remove_subscriber(pid))
    }

    pub(crate) fn get_ipc_message(&mut self, msg_topic: &str) -> Option<Vec<u8>> {
        let cur_pid: usize = unsafe { get_cur_task().pid };

        self.topic_lookup
            .get_mut(msg_topic)
            .and_then(|topic| topic.subscribers.get_mut(&cur_pid))
            .and_then(|subscriber| {
                let next_node: Arc<MessageNode> = match &subscriber.next_message {
                    Some(m) => Arc::clone(m),
                    None => return None,
                };
                subscriber.next_message = (*next_node.next.borrow()).as_ref().map(Arc::clone);
                Some(next_node.data.clone())
            })
    }

    pub(crate) fn get_subscriber_lock(&mut self, msg_topic: &str) -> Option<&Semaphore> {
        let cur_pid: usize = unsafe { get_cur_task().pid };
        self.topic_lookup
            .get_mut(msg_topic)
            .and_then(|topic| topic.subscribers.get_mut(&cur_pid))
            .map(|subscriber| &subscriber.lock)
    }
}
