#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use fe_osi;
use fe_rtos;
use rust_tm4c::gpio;
use rust_tm4c::interrupt;
use rust_tm4c::scb;
use rust_tm4c::system_control;
use rust_tm4c::systick;
use rust_tm4c::tm4c1294_peripherals;
use rust_tm4c::tm4c1294_peripherals::get_peripherals;
use rust_tm4c::uart;

const XTAL_FREQ: u32 = 25_000_000;
const CPU_FREQ: u32 = 120_000_000;

fn sysctl_server(sysctl_ptr: *const u32) {
    let sysctl = unsafe { &mut *(sysctl_ptr as *mut system_control::SystemControl) };
    loop {
        //pull a new message off the ipc queue
        //let msg = unsafe { Box::from_raw(fe_osi::ipc_receive(-1)) };
    }
}

fn scb_server(scb_ptr: *const u32) {
    let scb = unsafe { &mut *(scb_ptr as *mut scb::SystemControlBlock) };
    loop {
        //pull a new message off the ipc queue
        //let msg = unsafe { Box::from_raw(fe_osi::ipc_receive(-1)) };
    }
}

fn uart0_server(serial_ptr: *const u32) {
    let mut serial = unsafe { &mut *(serial_ptr as *mut uart::Uart) };
    let mut uart_driver = uart::drivers::UartBlockingDriver::new(&mut serial);
    loop {
        //let msg = fe_osi::ipc_receive(-1);
        for c in "Hello World\r\n".chars() {
            uart_driver.putchar(c);
        }
    }
}

fn gpion_server(gpion_ptr: *const u32) {
    let gpion = unsafe { &mut *(gpion_ptr as *mut gpio::Gpio) };
    loop {
        //pull a new message off the ipc queue
        //let msg = unsafe { Box::from_raw(fe_osi::ipc_receive(-1)) };
        gpion.set_low(gpio::Pin::Pin0);
        gpion.set_high(gpio::Pin::Pin1);
        let mut i = 200_000;
        while i > 0 {
            i = i - 1;
        }

        gpion.set_high(gpio::Pin::Pin0);
        gpion.set_low(gpio::Pin::Pin1);
        let mut i = 200_000;
        while i > 0 {
            i = i - 1;
        }
 
    }
}

#[no_mangle]
pub fn main() -> ! {
    let mut p = get_peripherals();
    let sysctl = p.take_system_control().unwrap();
    let gpion = p.take_gpion().unwrap();
    let gpioa = p.take_gpioa().unwrap();
    let systick = p.take_systick().unwrap();
    let scb = p.take_scb().unwrap();
    let uart0 = p.take_uart0().unwrap();

    // Configure the CPU for the maximum operating frequency
    let cpu_freq = sysctl.tm4c129_config_sysclk(CPU_FREQ, XTAL_FREQ);

    // Set up LEDs
    sysctl.enable_gpio_clock(system_control::GpioPort::GpioN);
    gpion.configure_as_output(gpio::Pin::Pin0);
    gpion.configure_as_output(gpio::Pin::Pin1);

    // Set up the debug UART
    sysctl.enable_gpio_clock(system_control::GpioPort::GpioA);
    sysctl.enable_uart_clock(system_control::Uart::Uart0);
    gpioa.select_alternate_function(gpio::Pin::Pin0, 1);
    gpioa.select_alternate_function(gpio::Pin::Pin1, 1);

    let _baud = uart0
        .configure(
            cpu_freq,
            115200,
            uart::Parity::None,
            uart::StopBits::One,
            uart::WordLength::Eight,
        )
        .unwrap();


    scb.int_register(&interrupt::IntType::SysTick, fe_rtos::task::sys_tick);
    scb.int_register(&interrupt::IntType::PendSV, fe_rtos::task::context_switch);
    scb.int_register(&interrupt::IntType::SVCall, fe_rtos::syscall::svc_handler);


    unsafe {
        fe_rtos::task::add_task(
            128,
            uart0_server,
            Some(uart0 as *const uart::Uart as *const u32),
        );
        fe_rtos::task::add_task(
            128,
            gpion_server,
            Some(gpion as *const gpio::Gpio as *const u32),
        );

    }
    fe_rtos::task::start_scheduler(rust_tm4c::tm4c1294_peripherals::trigger_pendsv);

    systick.enable_systick(120_000);
    unsafe { scb.SYSPRI3.modify(|x| x | (7 << 21)); }

    loop {}
}
