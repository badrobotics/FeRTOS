use core::sync::atomic::{AtomicUsize, Ordering};

#[repr(C)]
pub struct Semaphore {
    count: AtomicUsize,
    mutex: bool,
}

extern "C" {
    //System call to block until a semaphore is available
    fn do_block(sem: *const Semaphore) -> usize;
}

impl Semaphore {
    pub const fn new(start_count: usize) -> Semaphore {
        Semaphore {
            count: AtomicUsize::new(start_count),
            mutex: false,
        }
    }

    pub const fn new_mutex() -> Semaphore {
        Semaphore {
            count: AtomicUsize::new(1),
            mutex: true,
        }
    }

    pub fn is_available(&self) -> bool {
        if self.count.load(Ordering::Relaxed) > 0 {
            true
        } else {
            false
        }
    }

    pub fn take(&self) {
        loop {
            let old_val = self.count.load(Ordering::SeqCst);

            //If we cannot take the semaphore, block until we maybe can
            if old_val == 0 {
                unsafe { while do_block(self as *const Semaphore) != 0 {} }
                continue;
            }

            if self
                .count
                .compare_and_swap(old_val, old_val - 1, Ordering::SeqCst)
                == old_val
            {
                break;
            }
        }
    }

    pub fn give(&self) {
        if self.mutex {
            self.count.store(1, Ordering::SeqCst);
        } else {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
    }
}
