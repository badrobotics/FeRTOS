use crate::task::get_cur_task;
use core::sync::atomic::{AtomicUsize, Ordering};
use fe_osi::r#yield;

pub(crate) struct Spinlock {
    count: AtomicUsize,
    pid: AtomicUsize,
}

impl Spinlock {
    pub const fn new() -> Self {
        Spinlock {
            count: AtomicUsize::new(0),
            pid: AtomicUsize::new(0),
        }
    }

    pub fn take(&self) {
        let pid = unsafe { get_cur_task().pid };

        loop {
            if self.count.compare_and_swap(0, 1, Ordering::SeqCst) == 0 {
                self.pid.store(pid, Ordering::SeqCst);
                break;
            } else if pid == self.pid.load(Ordering::SeqCst) {
                self.count.fetch_add(1, Ordering::SeqCst);
                break;
            }

            r#yield();
        }
    }

    pub fn try_take(&self) -> bool {
        let pid = unsafe { get_cur_task().pid };

        //Don't check if our pid is the same as the lock's pid because interrupts
        //can screw with that.
        if self.count.compare_and_swap(0, 1, Ordering::SeqCst) == 0 {
            self.pid.store(pid, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub fn give(&self) {
        let pid = unsafe { get_cur_task().pid };

        if pid != self.pid.load(Ordering::SeqCst) {
            return;
        }

        if self.count.load(Ordering::SeqCst) <= 1 {
            self.pid.store(0, Ordering::SeqCst);
            self.count.store(0, Ordering::SeqCst);
        } else {
            self.count.fetch_sub(1, Ordering::SeqCst);
        }
    }
}
