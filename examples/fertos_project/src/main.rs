#![no_std]
#![no_main]

#[macro_use]
extern crate alloc;
extern crate atomic_queue;
extern crate fe_osi;
#[macro_use]
extern crate lazy_static;

use alloc::boxed::Box;
use atomic_queue::AtomicQueue;
use core::fmt::Write;
use cortex_m::peripheral::scb::Exception;
use fe_rtos;
use hal::prelude::*;
use rust_tm4c;
use tm4c129x_hal as hal;

static mut STORAGE: [char; 255] = [' '; 255];
lazy_static! {
    static ref QUEUE: AtomicQueue<'static, char> = {
        let m = unsafe { AtomicQueue::new(&mut STORAGE) };
        m
    };
}

fn hello_world(_: &mut u8) {
    let mut counter: u32 = 0;
    loop {
        let msg = format!("Hello, World! counter={}\r\n", counter);
        for c in msg.chars() {
            match QUEUE.push(c) {
                Err(_) => panic!("No room to push?"),
                Ok(_) => {}
            }
        }
        counter = counter.wrapping_add(1);
        fe_osi::sleep(10);
    }
}

fn uart_transmit_server<T: Write>(serial: &mut T) {
    loop {
        match QUEUE.pop() {
            Some(c) => {
                write!(serial, "{}", c).unwrap();
            }
            None => {
                fe_osi::sleep(10);
            }
        }
    }
}

#[no_mangle]
fn main() -> ! {
    let p = hal::Peripherals::take().unwrap();
    let cp = hal::CorePeripherals::take().unwrap();

    let mut sc = p.SYSCTL.constrain();
    sc.clock_setup.oscillator = hal::sysctl::Oscillator::Main(
        hal::sysctl::CrystalFrequency::_25mhz,
        hal::sysctl::SystemClock::UsePll(hal::sysctl::PllOutputFrequency::_120mhz),
    );
    let clocks = sc.clock_setup.freeze();

    let mut porta = p.GPIO_PORTA_AHB.split(&sc.power_control);

    // Activate UART
    let uart0 = hal::serial::Serial::uart0(
        p.UART0,
        porta
            .pa1
            .into_af_push_pull::<hal::gpio::AF1>(&mut porta.control),
        porta
            .pa0
            .into_af_push_pull::<hal::gpio::AF1>(&mut porta.control),
        (),
        (),
        115200_u32.bps(),
        hal::serial::NewlineMode::SwapLFtoCRLF,
        &clocks,
        &sc.power_control,
    );

    rust_tm4c::interrupt::int_register(
        (Exception::SysTick.irqn() + 16) as u32,
        fe_rtos::task::sys_tick,
    );
    rust_tm4c::interrupt::int_register(
        (Exception::PendSV.irqn() + 16) as u32,
        fe_rtos::task::context_switch,
    );
    rust_tm4c::interrupt::int_register(
        (Exception::SVCall.irqn() + 16) as u32,
        fe_rtos::syscall::svc_handler,
    );

    fe_osi::task::task_spawn(
        fe_rtos::task::DEFAULT_STACK_SIZE,
        uart_transmit_server,
        Some(Box::new(uart0)),
    );

    fe_osi::task::task_spawn(fe_rtos::task::DEFAULT_STACK_SIZE, hello_world, None);

    let reload_val: u32 = cortex_m::peripheral::SYST::get_ticks_per_10ms() / 10;

    // Start the FeRTOS scheduler
    fe_rtos::task::start_scheduler(cortex_m::peripheral::SCB::set_pendsv, cp.SYST, reload_val);

    loop {}
}
