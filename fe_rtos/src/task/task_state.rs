use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, Ordering};
use fe_osi::semaphore::Semaphore;

#[derive(Clone, Copy)]
pub(crate) enum TaskState {
    Runnable,
    Asleep(u64),
    Blocking(*const Semaphore),
    Zombie,
}

pub(crate) struct TaskStateStruct {
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

        if self
            .in_use
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            ret_val = Some(*self.state.borrow());
            self.in_use.store(false, Ordering::SeqCst);
        }

        ret_val
    }

    pub fn try_set(&self, new_state: TaskState) -> bool {
        if self
            .in_use
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            self.state.replace(new_state);
            self.in_use.store(false, Ordering::SeqCst);
            //If the task state changes from being runnable, yield to make
            //sure the task doesn't run when it shouldn't
            match new_state {
                TaskState::Runnable => (),
                _ => {
                    fe_osi::r#yield();
                }
            }

            true
        } else {
            false
        }
    }
}

unsafe impl Send for TaskStateStruct {}
unsafe impl Sync for TaskStateStruct {}
