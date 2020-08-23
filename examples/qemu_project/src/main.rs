#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use alloc::sync::Arc;
use fe_osi::allocator::get_heap_remaining;

fn write_byte(c: u8) {
    let uart0: *mut usize = 0x4000_C000 as *mut usize;
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
        // Give the subscribers enough time to clear the entries
        fe_osi::sleep(50);
    }
}

fn writer_task(_: &mut usize) {
    let mut subscriber = fe_osi::ipc::Subscriber::new("stdout").unwrap();
    loop {
        if let Some(msg) = subscriber.get_message_nonblocking() {
            for c in msg {
                write_byte(c);
            }
        }
    }
}

fn test_task(_: &mut usize) {
    {
        let _test: Arc<[u32]> = Arc::new([0; 200]);
        let _test2: Box<[u32]> = Box::new([0; 200]);
        let _subscriber = fe_osi::ipc::Subscriber::new("stdout").unwrap();
        fe_osi::sleep(1000);
    }
    unsafe {
        fe_osi::exit();
    }
    loop {}
}

fn spawn_task(_: &mut usize) {
    loop {
        fe_osi::task::task_spawn_block(fe_rtos::task::DEFAULT_STACK_SIZE, test_task, None);
        fe_osi::sleep(1000);
    }
}

#[no_mangle]
fn main() -> ! {
    let mut p = cortex_m::peripheral::Peripherals::take().unwrap();

    fe_rtos::arch::arch_setup(&mut p);

    fe_osi::task::task_spawn(fe_rtos::task::DEFAULT_STACK_SIZE, hello_task, None);
    fe_osi::task::task_spawn(fe_rtos::task::DEFAULT_STACK_SIZE, writer_task, None);
    fe_osi::task::task_spawn(fe_rtos::task::DEFAULT_STACK_SIZE, spawn_task, None);

    fe_osi::set_putc(|c: char| {
        write_byte(c as u8);
    });

    //Start the FeRTOS scheduler
    let enable_systick = |reload: usize| {
        p.SYST.set_reload(reload as u32);
        p.SYST.clear_current();
        p.SYST.enable_counter();
        p.SYST.enable_interrupt();
    };
    let reload_val = cortex_m::peripheral::SYST::get_ticks_per_10ms() / 10;
    fe_rtos::task::start_scheduler(enable_systick, reload_val as usize);

    loop {}
}
