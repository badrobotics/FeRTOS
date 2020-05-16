#![no_std]
#![no_main]

extern crate alloc;

use cortex_m::peripheral::scb::Exception;

fn write_byte(c : u8) {
    let uart0 : *mut usize = 0x4000_C000 as *mut usize;
    unsafe {
        *uart0 = c as usize;
    }
}

fn hello_task(_ : &mut usize) {
    let mut stdout = fe_osi::ipc::Publisher::new("stdout");
    let mut counter = 0;
    loop {
<<<<<<< HEAD
        let msg = alloc::format!("Hello, World! {} {:X}\r\n", counter, fe_osi::allocator::get_heap_remaining()).into_bytes();
        stdout.publish(msg).unwrap();
=======
        let msg = alloc::format!("Hello, World! {}\r\n", counter).into_bytes();
        lock.with_lock(|| {
            for c in &msg {
                write_byte(*c);
            }
        });
>>>>>>> master
        counter += 1;
    }
}

fn writer_task(_ : &mut usize) {
    let mut subscriber = fe_osi::ipc::Subscriber::new("stdout").unwrap();
    loop {
<<<<<<< HEAD
        if let Some(message) = subscriber.get_message() {
            for c in message {
                write_byte(c);
            }
        }
=======
        let msg = alloc::format!("Welcome to FeRTOS! {}\r\n", counter).into_bytes();
        lock.with_lock(|| {
            for c in &msg {
                write_byte(*c);
            }
        });
        counter += 1;
        fe_osi::sleep(10);
>>>>>>> master
    }
}

#[no_mangle]
fn main() -> ! {
    let p = cortex_m::peripheral::Peripherals::take().unwrap();

    fe_rtos::interrupt::int_register(Exception::SysTick.irqn(), fe_rtos::task::sys_tick);
    fe_rtos::interrupt::int_register(Exception::PendSV.irqn(), fe_rtos::task::context_switch);
    fe_rtos::interrupt::int_register(Exception::SVCall.irqn(), fe_rtos::syscall::svc_handler);

    fe_osi::task::task_spawn(fe_rtos::task::DEFAULT_STACK_SIZE, hello_task, None);
    fe_osi::task::task_spawn(fe_rtos::task::DEFAULT_STACK_SIZE, writer_task, None);

    //Start the FeRTOS scheduler
    let reload_val : u32 = cortex_m::peripheral::SYST::get_ticks_per_10ms() / 10;
    fe_rtos::task::start_scheduler(cortex_m::peripheral::SCB::set_pendsv, p.SYST, reload_val);

    loop {}
}
