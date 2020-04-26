use fe_osi::semaphore::Semaphore;

pub(crate) struct Subscriber {
    pub(crate) lock: &'static Semaphore,
    pub(crate) index: usize,
}

impl Subscriber {
    pub(crate) fn new(sem: &'static Semaphore, index: Option<usize>) -> Subscriber {
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