#![no_std]
#![no_main]

extern crate alloc;

use alloc::sync::Arc;
use alloc::boxed::Box;
use cortex_m::peripheral::scb::Exception;
use fe_osi::semaphore::Semaphore;

fn write_byte(c : u8) {
    let uart0 : *mut usize = 0x4000_C000 as *mut usize;
    unsafe {
        *uart0 = c as usize;
    }
}

fn hello_task(lock : &mut Arc<Semaphore>) {
    let mut counter = 0;
    loop {
        let msg = alloc::format!("Hello, World! {}\r\n", counter).into_bytes();
        lock.take();
        for c in msg {
            write_byte(c);
        }
        lock.give();
        counter += 1;
        fe_osi::sleep(10);
    }
}

fn welcome_task(lock : &mut Arc<Semaphore>) {
    let mut counter = 0;
    loop {
        let msg = alloc::format!("Welcome to FeRTOS! {}\r\n", counter).into_bytes();
        lock.take();
        for c in msg {
            write_byte(c);
        }
        lock.give();
        counter += 1;
        fe_osi::sleep(10);
    }
}

#[no_mangle]
fn main() -> ! {
    let p = cortex_m::peripheral::Peripherals::take().unwrap();

    let lock = Arc::new(Semaphore::new_mutex());

    fe_rtos::interrupt::int_register(Exception::SysTick.irqn(), fe_rtos::task::sys_tick);
    fe_rtos::interrupt::int_register(Exception::PendSV.irqn(), fe_rtos::task::context_switch);
    fe_rtos::interrupt::int_register(Exception::SVCall.irqn(), fe_rtos::syscall::svc_handler);

    fe_osi::task::task_spawn(fe_rtos::task::DEFAULT_STACK_SIZE, hello_task, Some(Box::new(Arc::clone(&lock))));
    fe_osi::task::task_spawn(fe_rtos::task::DEFAULT_STACK_SIZE, welcome_task, Some(Box::new(lock)));

    //Start the FeRTOS scheduler
    let reload_val : u32 = cortex_m::peripheral::SYST::get_ticks_per_10ms() / 10;
    fe_rtos::task::start_scheduler(cortex_m::peripheral::SCB::set_pendsv, p.SYST, reload_val);

    loop {}
}
