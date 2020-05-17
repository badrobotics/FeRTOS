extern crate alloc;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cell::RefCell;
use fe_osi::semaphore::Semaphore;

pub(crate) struct MessageNode {
    pub data: Vec<u8>,
    pub next: RefCell<Option<Arc<MessageNode>>>,
}

pub(crate) struct Subscriber {
    pub(crate) lock: Semaphore,
    pub(crate) next_message: Option<Arc<MessageNode>>,
}

impl Subscriber {
    pub(crate) fn new(sem: Semaphore, first_message: Option<Arc<MessageNode>>) -> Subscriber {
        Subscriber {
            lock: sem,
            next_message: first_message,
        }
    }
}
