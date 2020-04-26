extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
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
    pub(crate) fn publish_to_topic(&mut self, message_topic: String, message: &Vec<u8>) {
        self.lock.take();
        for topic in &mut self.topic_lookup {
            if topic.name == message_topic {
                topic.data.push(message.clone());
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
            if topic.name == subscriber_topic {
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
    ) -> Option<Vec<u8>> {
        for topic in &mut self.topic_lookup {
            if topic.name == msg_topic {
                for subscriber in &mut topic.subscribers {
                    if subscriber.lock as *const Semaphore == sem as *const Semaphore {
                        let message: Vec<u8> = topic.data[subscriber.index].clone();
                        subscriber.index += 1;
                        return Some(message);
                    }
                }
            }
        }
        return None;
    }
}
