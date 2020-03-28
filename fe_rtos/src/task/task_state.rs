use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, Ordering};
use fe_osi::semaphore::Semaphore;

#[derive(Clone, Copy)]
pub enum TaskState {
    Runnable,
    Asleep(u64),
    Blocking(*const Semaphore),
    Zombie,
}

pub struct TaskStateStruct {
    state: RefCell<TaskState>,
    in_use: AtomicBool,
}

impl TaskStateStruct {
    pub const fn new() -> Self {
        TaskStateStruct {
            state: RefCell::new(TaskState::Runnable),
            in_use: AtomicBool::new(false),
        }
    }

    pub fn try_get(&self) -> Option<TaskState> {
        let mut ret_val = None;

        if self.in_use.compare_and_swap(false, true, Ordering::SeqCst) == false {
            ret_val = Some(*self.state.borrow());
            self.in_use.store(false, Ordering::SeqCst);
        }

        ret_val
    }

    pub fn try_set(&self, new_state: TaskState) -> bool {
        if self.in_use.compare_and_swap(false, true, Ordering::SeqCst) == false {
            self.state.replace(new_state);
            self.in_use.store(false, Ordering::SeqCst);

            true
        } else {
            false
        }
    }
}

unsafe impl Send for TaskStateStruct {}
unsafe impl Sync for TaskStateStruct {}
