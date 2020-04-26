mod subscriber;
mod topic;

extern crate alloc;
use crate::ipc::subscriber::Subscriber;
use crate::ipc::topic::Topic;
use alloc::string::String;
use alloc::vec::Vec;
use fe_osi::semaphore::Semaphore;

pub(crate) struct TopicRegistry {
    pub(crate) lock: Semaphore,
    pub(crate) topic_lookup: Vec<Topic>,
}

pub(crate) static mut TOPIC_REGISTERY: TopicRegistry = TopicRegistry {
    lock: Semaphore::new_mutex(),
    topic_lookup: Vec::new(),
};

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

        // If the topic already exists in the registery, add a new subscriber
        let mut topic_exists = false;
        for topic in &mut self.topic_lookup {
            if topic.name == subscriber_topic {
                topic_exists = true;
                let subscriber = Subscriber::new(sem, Some(topic.data.len()));

                // take the condition variable to indicate no new messages
                subscriber.lock.take();
                topic.subscribers.push(subscriber);
            }
        }
        // If the topic already exists in the registery, add a new subscriber
        if !topic_exists {
            // create the new topic
            let mut new_topic = Topic::new(&subscriber_topic);

            // create a subscriber to add to it
            let subscriber = Subscriber::new(sem, None);
            // take the lock
            subscriber.lock.take();

            // add subscriber to topic subscriber list
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
