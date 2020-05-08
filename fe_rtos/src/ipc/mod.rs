mod subscriber;
mod topic;

extern crate alloc;
use crate::ipc::subscriber::Subscriber;
use crate::ipc::topic::Topic;
use alloc::string::String;
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
    fn topic_exists(&mut self, requested_topic: &String) -> bool {
        for topic in &mut self.topic_lookup {
            if topic.name == *requested_topic {
                return true;
            }
        }
        return false;
    }

    fn add_topic(&mut self, topic: Topic) {
        self.topic_lookup.push(topic);
    }

    pub(crate) fn publish_to_topic(&mut self, message_topic: String, message: &Vec<u8>) {
        for topic in &mut self.topic_lookup {
            if topic.name == message_topic {
                topic.cleanup();
                topic.add_message(message);
            }
        }
    }

    pub(crate) fn subscribe_to_topic(&mut self, subscriber_topic: String, sem: Semaphore) -> &Semaphore {
        let pid: usize = unsafe { get_cur_task().pid };
        let subscriber = Subscriber::new(sem, None);

        if !self.topic_exists(&subscriber_topic) {
            let new_topic = Topic::new(&subscriber_topic);
            self.add_topic(new_topic);
        }
        
        // get reference to desired topic 
        let topic: &mut Topic = {
            let mut ret = None;
            for topic in &mut self.topic_lookup {
                if topic.name == *subscriber_topic {
                    ret = Some(topic);
                }
            }
            ret
        }.unwrap();

        topic.add_subscriber(pid, subscriber);

        &topic.subscribers.get(&pid).unwrap().lock
    }

    pub(crate) fn get_ipc_message(
        &mut self,
        msg_topic: String,
    ) -> Option<Vec<u8>> {
        let cur_pid: usize = unsafe { get_cur_task().pid };
        for topic in &mut self.topic_lookup {
            if topic.name == msg_topic {
                match topic.subscribers.get_mut(&cur_pid) {
                    Some(subscriber) => {
                        let message: Vec<u8> = topic.data[subscriber.index].clone();
                        subscriber.index += 1;
                        return Some(message);
                    },
                    None => {},
                }
            }
        }
        return None;
    }
}
