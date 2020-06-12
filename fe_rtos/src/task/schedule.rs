extern crate alloc;

use crate::task::task_state::TaskState;
use crate::task::Task;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub(crate) trait Scheduler {
    fn add_task(&mut self, new_task: Arc<Task>);
    fn next(&mut self) -> Option<Arc<Task>>;
}

pub(crate) struct RoundRobin {
    queue: Vec<Arc<Task>>,
    current_index: AtomicUsize,
    pushing_task: AtomicBool,
}

impl RoundRobin {
    pub const fn new() -> Self {
        RoundRobin {
            queue: Vec::new(),
            current_index: AtomicUsize::new(0),
            pushing_task: AtomicBool::new(false),
        }
    }

    fn increment_index(&self) {
        let idx = self.current_index.load(Ordering::SeqCst);
        if idx == usize::MAX || idx + 1 >= self.queue.len() {
            self.current_index.store(0, Ordering::SeqCst);
        } else {
            self.current_index.fetch_add(1, Ordering::SeqCst);
        }
    }
}

impl Scheduler for RoundRobin {
    fn add_task(&mut self, new_task: Arc<Task>) {
        self.pushing_task.store(true, Ordering::SeqCst);
        self.queue.push(new_task);
        self.pushing_task.store(false, Ordering::SeqCst);
    }

    fn next(&mut self) -> Option<Arc<Task>> {
        //Because the scheduler function can't block, return None if something
        //is being pushed to the queue
        if self.pushing_task.load(Ordering::SeqCst) {
            return None;
        }
        if self.queue.is_empty() {
            return None;
        }

        //Loop until we find a runnable task. This is fine because the kernel
        //should always be runnable.
        loop {
            let idx = self.current_index.load(Ordering::SeqCst);
            let len = self.queue.len();
            if idx >= len {
                self.increment_index();
                continue;
            }
            let task = &self.queue[idx];
            let state = task.state.try_get().unwrap_or(TaskState::Runnable);

            match state {
                TaskState::Runnable => {
                    self.increment_index();
                    return Some(Arc::clone(&task));
                }
                TaskState::Zombie => {
                    self.queue.swap_remove(idx);

                    if idx != 0 {
                        self.current_index.fetch_sub(1, Ordering::SeqCst);
                    }
                }
                _ => (),
            };

            self.increment_index();
        }
    }
}
