use fe_osi::semaphore::Semaphore;

pub(crate) struct Subscriber {
    pub(crate) lock: Semaphore,
    pub(crate) index: usize,
}

impl Subscriber {
    pub(crate) fn new(sem: Semaphore, index: Option<usize>) -> Subscriber {
        let start_index: usize = match index {
            Some(i) => i,
            None => 0,
        };
        Subscriber {
            lock: sem,
            index: start_index,
        }
    }
}