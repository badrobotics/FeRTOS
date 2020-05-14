mod subscriber;
mod topic;

extern crate alloc;
use crate::ipc::subscriber::Subscriber;
use crate::ipc::topic::Topic;
use alloc::vec::Vec;
use fe_osi::semaphore::Semaphore;
use crate::task::get_cur_task;

pub(crate) struct TopicRegistry {
    pub(crate) topic_lookup: Vec<Topic>,
}

pub(crate) static mut TOPIC_REGISTERY_LOCK: Semaphore = Semaphore::new_mutex();
pub(crate) static mut TOPIC_REGISTERY: TopicRegistry = TopicRegistry {
    topic_lookup: Vec::new(),
};

impl TopicRegistry {
    fn topic_exists(&mut self, requested_topic: &str) -> bool {
        for topic in &mut self.topic_lookup {
            if topic.name == *requested_topic {
                return true;
            }
        }
        false
    }

    fn add_topic(&mut self, topic: Topic) {
        self.topic_lookup.push(topic);
    }

    pub(crate) fn publish_to_topic(&mut self, message_topic: &str, message: &[u8]) {
        for topic in &mut self.topic_lookup {
            if topic.name == message_topic {
                topic.cleanup();
                topic.add_message(message);
            }
        }
    }

    pub(crate) fn subscribe_to_topic(&mut self, subscriber_topic: &str) -> Option<&Semaphore> {
        let sem: Semaphore = Semaphore::new(0);
        let pid: usize = unsafe { get_cur_task().pid };
        let subscriber = Subscriber::new(sem, None);

        if !self.topic_exists(&subscriber_topic) {
            let new_topic = Topic::new(&subscriber_topic);
            self.add_topic(new_topic);
        }
        
        // get reference to desired topic 
        let topic = {
            let mut ret = None;
            for topic in &mut self.topic_lookup {
                if topic.name == *subscriber_topic {
                    ret = Some(topic);
                }
            }
            ret
        };

        match topic {
            Some(t) => {
                t.add_subscriber(pid, subscriber);
                match &t.subscribers.get(&pid) {
                    Some(subscriber) => Some(&subscriber.lock),
                    None => None,
                }
            },
            None => None,
        }
    }

    pub(crate) fn get_ipc_message(
        &mut self,
        msg_topic: &str,
    ) -> Option<Vec<u8>> {
        let cur_pid: usize = unsafe { get_cur_task().pid };
        for topic in &mut self.topic_lookup {
            if topic.name == msg_topic {
                if let Some(subscriber) = topic.subscribers.get_mut(&cur_pid) {
                    let message: Vec<u8> = topic.data[subscriber.index].clone();
                    subscriber.index += 1;
                    return Some(message);
                }
            }
        }
        None
    }
}
