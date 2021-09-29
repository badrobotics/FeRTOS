use core::sync::atomic::{AtomicUsize, Ordering};

/// A tool used to prevent data races.
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
    /// Creates a Semaphore with a specified starting count.
    pub const fn new(start_count: usize) -> Semaphore {
        Semaphore {
            count: AtomicUsize::new(start_count),
            mutex: false,
        }
    }

    /// Creates a mutex Semaphore. Usually used for locking
    pub const fn new_mutex() -> Semaphore {
        Semaphore {
            count: AtomicUsize::new(1),
            mutex: true,
        }
    }

    /// Returns true if the Semaphore can be taken, false otherwise.
    pub fn is_available(&self) -> bool {
        self.count.load(Ordering::Relaxed) > 0
    }

    /// Takes a Semaphore for a task. If the Semaphore cannot be taken right
    /// away, block until it can be taken.
    pub fn take(&self) {
        loop {
            let old_val = self.count.load(Ordering::SeqCst);

            //If we cannot take the semaphore, block until we maybe can
            if old_val == 0 {
                unsafe {
                    do_block(self as *const Semaphore);
                }
                continue;
            }

            if self
                .count
                .compare_exchange(old_val, old_val - 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                break;
            }
        }
    }

    /// Attempt to take the Semaphore. If the Semaphore is taken, return true.
    /// Otherwise, return false.
    pub fn try_take(&self) -> bool {
        let old_val = self.count.load(Ordering::SeqCst);

        if old_val == 0 {
            return false;
        }

        self.count
            .compare_exchange(old_val, old_val - 1, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    /// In a mutex Semaphore, give() sets the count to 1.
    /// In a non-mutex Semaphore, gives increments the count by 1.
    pub fn give(&self) {
        if self.mutex {
            self.count.store(1, Ordering::SeqCst);
        } else {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// Takes the Semaphore, executes the closure, then gives the Semaphore.
    pub fn with_lock<F: FnMut()>(&self, mut f: F) {
        self.take();
        f();
        self.give();
    }

    /// Attempts to take the Semaphore. If it succeeds, execute the closure, return
    /// true, then give back the Semaphore. Otherwise, it will return false.
    pub fn try_with_lock<F: FnMut()>(&self, mut f: F) -> bool {
        if self.try_take() {
            f();
            self.give();
            true
        } else {
            false
        }
    }
}
