#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use alloc::sync::Arc;
use fe_osi::allocator::get_heap_remaining;

fn write_byte(c: u8) {
    let uart0: *mut usize = 0x1000_0000 as *mut usize;
    unsafe {
        *uart0 = c as usize;
    }
}

fn hello_task(_: &mut usize) {
    let mut stdout = fe_osi::ipc::Publisher::new("stdout").unwrap();
    let mut counter = 0;
    loop {
        let msg =
            alloc::format!("Hello, World! {} {:X}\r\n", counter, get_heap_remaining()).into_bytes();
        stdout.publish(msg).unwrap();
        counter += 1;
        fe_osi::sleep(5);
    }
}

fn writer_task(_: &mut usize) {
    let mut subscriber = fe_osi::ipc::Subscriber::new("stdout").unwrap();
    loop {
        if let Some(msg) = subscriber.get_message_nonblocking() {
            for c in msg {
                write_byte(c);
            }
        } else {
            fe_osi::r#yield();
        }
    }
}

fn test_task(_: &mut usize) {
    {
        let _test: Arc<[u32]> = Arc::new([0; 200]);
        let _test2: Box<[u32]> = Box::new([0; 200]);
        let _subscriber = fe_osi::ipc::Subscriber::new("stdout").unwrap();
        fe_osi::sleep(100);
    }
    unsafe {
        fe_osi::exit();
    }
    loop {}
}

fn spawn_task(_: &mut usize) {
    loop {
        fe_osi::task::task_spawn_block(fe_rtos::task::DEFAULT_STACK_SIZE, test_task, None);
        fe_osi::sleep(100);
    }
}

#[no_mangle]
fn main() -> ! {
    const MTIME: *const u64 = 0x200BFF8 as *const u64;
    const MTIMECMP: *mut u64 = 0x2004000 as *mut u64;
    const TIMEBASE_FREQ: usize = 10000000;

    unsafe {
        fe_rtos::arch::arch_setup(MTIME, MTIMECMP);
    }

    fe_osi::set_putc(|c: char| {
        write_byte(c as u8);
    });

    fe_osi::task::task_spawn(fe_rtos::task::DEFAULT_STACK_SIZE, hello_task, None);
    fe_osi::task::task_spawn(fe_rtos::task::DEFAULT_STACK_SIZE, writer_task, None);
    fe_osi::task::task_spawn(fe_rtos::task::DEFAULT_STACK_SIZE, spawn_task, None);

    //Start the FeRTOS scheduler
    //Systick every 10ms
    let reload_val = TIMEBASE_FREQ / 100;
    fe_rtos::task::start_scheduler(fe_rtos::arch::enable_systick, reload_val);

    loop {}
}
