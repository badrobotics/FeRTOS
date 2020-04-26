extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use fe_osi::ipc::Message;
use fe_osi::semaphore::Semaphore;

pub(crate) struct Subscriber {
    lock: &'static Semaphore,
    index: usize,
}

pub(crate) struct TopicRegistry {
    pub(crate) lock: Semaphore,
    pub(crate) topic_lookup: Vec<Topic>,
}

pub(crate) static mut TOPIC_REGISTERY: TopicRegistry = TopicRegistry {
    lock: Semaphore::new_mutex(),
    topic_lookup: Vec::new(),
};

pub(crate) struct Topic {
    name: String,
    data: Vec<Vec<u8>>,
    subscribers: Vec<Subscriber>,
}

impl TopicRegistry {
    pub(crate) fn publish_to_topic(&mut self, message_topic: String, message: Message) {
        self.lock.take();
        let msg_vec: Vec<u8> = message.into();
        for topic in &mut self.topic_lookup {
            if topic.name.trim() == message_topic.trim() {
                let msg_clone = msg_vec.clone();
                topic.data.push(msg_clone);
                for subscriber in &mut topic.subscribers {
                    subscriber.lock.give();
                }
            }
        }
        self.lock.give();
    }

    pub(crate) fn subscribe_to_topic(&mut self, subscriber_topic: String, sem: &'static Semaphore) {
        self.lock.take();
        let mut topic_exists = false;
        for topic in &mut self.topic_lookup {
            if topic.name.trim() == subscriber_topic.trim() {
                topic_exists = true;
                let subscriber = Subscriber {
                    lock: sem,
                    index: topic.data.len(),
                };
                subscriber.lock.take();
                topic.subscribers.push(subscriber);
            }
        }
        if !topic_exists {
            let mut new_topic = Topic {
                name: subscriber_topic.clone(),
                data: Vec::new(),
                subscribers: Vec::new(),
            };
            let subscriber = Subscriber {
                lock: sem,
                index: 0,
            };
            subscriber.lock.take();
            new_topic.subscribers.push(subscriber);
            self.topic_lookup.push(new_topic);
        }
        self.lock.give();
    }

    pub(crate) fn get_ipc_message(
        &mut self,
        msg_topic: String,
        sem: &'static Semaphore,
    ) -> Option<Message> {
        for topic in &mut self.topic_lookup {
            if topic.name == msg_topic {
                for subscriber in &mut topic.subscribers {
                    if subscriber.lock as *const Semaphore == sem as *const Semaphore {
                        let msg_vec: Vec<u8> = topic.data[subscriber.index].clone();
                        subscriber.index += 1;
                        return Some(msg_vec.into());
                    }
                }
            }
        }
        return None;
    }
}
