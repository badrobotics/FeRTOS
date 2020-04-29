#![no_std]
#![no_main]

mod cmd;
mod uart_server;
mod stdio;

extern crate alloc;

use alloc::boxed::Box;
use hal::prelude::*;
use tm4c129x_hal as hal;
use cortex_m::peripheral::scb::Exception;
use fe_osi;
use fe_rtos;

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

    fe_rtos::interrupt::int_register(Exception::SysTick.irqn(), fe_rtos::task::sys_tick);
    fe_rtos::interrupt::int_register(Exception::PendSV.irqn(), fe_rtos::task::context_switch);
    fe_rtos::interrupt::int_register(Exception::SVCall.irqn(), fe_rtos::syscall::svc_handler);

    let (uart0_tx, uart0_rx) = uart0.split();

    fe_osi::task::task_spawn(
        fe_rtos::task::DEFAULT_STACK_SIZE,
        uart_server::uart_transmit_server,
        Some(Box::new(uart0_tx)),
    );

    fe_osi::task::task_spawn(
        fe_rtos::task::DEFAULT_STACK_SIZE,
        uart_server::uart_receive_server,
        Some(Box::new(uart0_rx)),
    );

    fe_osi::task::task_spawn(fe_rtos::task::DEFAULT_STACK_SIZE, cmd::cmd, None);
    fe_osi::task::task_spawn(fe_rtos::task::DEFAULT_STACK_SIZE, cmd::hello_world, None);

    // Start the FeRTOS scheduler
    let reload_val: u32 = cortex_m::peripheral::SYST::get_ticks_per_10ms() / 10;
    fe_rtos::task::start_scheduler(cortex_m::peripheral::SCB::set_pendsv, cp.SYST, reload_val);

    loop {}
}
