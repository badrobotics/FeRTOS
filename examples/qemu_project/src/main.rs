#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use alloc::sync::Arc;
use cortex_m::peripheral::scb::Exception;
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
    }
}

fn writer_task(_: &mut usize) {
    let mut subscriber = fe_osi::ipc::Subscriber::new("stdout").unwrap();
    loop {
        let msg = subscriber.get_message().unwrap();

        for c in msg {
            write_byte(c);
        }
    }
}

fn test_task(_: &mut usize) {
    let _test: Arc<[u32]> = Arc::new([0; 200]);
    let _test2: Box<[u32]> = Box::new([0; 200]);
    fe_osi::sleep(1000);
    fe_osi::exit();
    loop {}
}

fn spawn_task(_: &mut usize) {
    loop {
        fe_osi::task::task_spawn(fe_rtos::task::DEFAULT_STACK_SIZE, test_task, None);
        fe_osi::sleep(2000);
    }
}

#[no_mangle]
fn main() -> ! {
    let mut p = cortex_m::peripheral::Peripherals::take().unwrap();

    fe_rtos::interrupt::int_register(Exception::SysTick.irqn(), fe_rtos::task::sys_tick);
    fe_rtos::interrupt::int_register(Exception::PendSV.irqn(), fe_rtos::task::context_switch);
    fe_rtos::interrupt::int_register(Exception::SVCall.irqn(), fe_rtos::syscall::svc_handler);

    fe_osi::task::task_spawn(fe_rtos::task::DEFAULT_STACK_SIZE, hello_task, None);
    fe_osi::task::task_spawn(fe_rtos::task::DEFAULT_STACK_SIZE, writer_task, None);
    fe_osi::task::task_spawn(fe_rtos::task::DEFAULT_STACK_SIZE, spawn_task, None);

    //It's probably a good idea to have the context switch be the lowest
    //priority interrupt.
    unsafe {
        p.SCB
            .set_priority(cortex_m::peripheral::scb::SystemHandler::PendSV, 7);
    }

    //Start the FeRTOS scheduler
    let enable_systick = |reload: usize| {
        p.SYST.set_reload(reload as u32);
        p.SYST.clear_current();
        p.SYST.enable_counter();
        p.SYST.enable_interrupt();
    };
    let reload_val = cortex_m::peripheral::SYST::get_ticks_per_10ms() / 10;
    fe_rtos::task::start_scheduler(
        cortex_m::peripheral::SCB::set_pendsv,
        enable_systick,
        reload_val as usize,
    );

    loop {}
}
