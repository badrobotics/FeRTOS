mod subscriber;
mod topic;

extern crate alloc;
use crate::ipc::subscriber::MessageNode;
use crate::ipc::subscriber::Subscriber;
use crate::ipc::topic::Topic;
use crate::task::get_cur_task;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use fe_osi::semaphore::Semaphore;

pub(crate) struct TopicRegistry<'a> {
    pub(crate) topic_lookup: BTreeMap<&'a str, Topic>,
}

pub(crate) static mut TOPIC_REGISTERY_LOCK: Semaphore = Semaphore::new_mutex();
pub(crate) static mut TOPIC_REGISTERY: TopicRegistry = TopicRegistry {
    topic_lookup: BTreeMap::new(),
};

impl<'a> TopicRegistry<'a> {
    pub(crate) fn publish_to_topic(&mut self, message_topic: &'a str, message: &[u8]) {
        self.topic_lookup
            .entry(message_topic)
            .and_modify(|topic| topic.add_message(message));
    }

    pub(crate) fn subscribe_to_topic(&mut self, subscriber_topic: &'a str) {
        let sem: Semaphore = Semaphore::new(0);
        let pid: usize = unsafe { get_cur_task().pid };
        let subscriber = Subscriber::new(sem, None);

        self.topic_lookup
            .entry(subscriber_topic)
            .or_insert_with(Topic::new)
            .add_subscriber(pid, subscriber);
    }

    pub(crate) fn get_ipc_message(&mut self, msg_topic: &str) -> Option<Vec<u8>> {
        let cur_pid: usize = unsafe { get_cur_task().pid };

        self.topic_lookup
            .get_mut(msg_topic)
            .and_then(|topic| topic.subscribers.get_mut(&cur_pid))
            .and_then(|subscriber| {
                let next_node: Arc<MessageNode> = match &subscriber.next_message {
                    Some(m) => Arc::clone(&m),
                    None => return None,
                };
                subscriber.next_message = match &*next_node.next.borrow() {
                    Some(m) => Some(Arc::clone(&m)),
                    None => None,
                };
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

    pub(crate) fn get_topic_lock(&mut self, topic_name: &str) -> Option<&Semaphore> {
        self.topic_lookup
            .get_mut(topic_name)
            .map(|topic| &topic.lock)
    }
}
