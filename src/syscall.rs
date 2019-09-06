use super::task;

extern "C" {
    pub fn svc_handler();
}

#[no_mangle]
extern "C" fn sys_exit() {
    unsafe { task::remove_task(); }
}

#[no_mangle]
extern "C" fn sys_sleep(ms32: u32) {
    let ms: u64 = ms32 as u64;
    task::sleep(ms);
}

