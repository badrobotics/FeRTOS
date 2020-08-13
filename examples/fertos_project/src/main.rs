#![no_std]
#![no_main]

mod cmd;
mod stdio;
mod uart_server;

#[macro_use]
extern crate alloc;

use alloc::boxed::Box;
use hal::prelude::*;
#[cfg(feature = "tm4c123")]
use tm4c123x_hal as hal;
#[cfg(feature = "tm4c1294")]
use tm4c129x_hal as hal;

#[cfg(feature = "tm4c1294")]
fn get_oscillator() -> hal::sysctl::Oscillator {
    hal::sysctl::Oscillator::Main(
        hal::sysctl::CrystalFrequency::_25mhz,
        hal::sysctl::SystemClock::UsePll(hal::sysctl::PllOutputFrequency::_120mhz),
    )
}

#[cfg(feature = "tm4c123")]
fn get_oscillator() -> hal::sysctl::Oscillator {
    hal::sysctl::Oscillator::Main(
        hal::sysctl::CrystalFrequency::_16mhz,
        hal::sysctl::SystemClock::UsePll(hal::sysctl::PllOutputFrequency::_80_00mhz),
    )
}

#[no_mangle]
fn main() -> ! {
    let p = hal::Peripherals::take().unwrap();
    let mut cp = hal::CorePeripherals::take().unwrap();

    let mut sc = p.SYSCTL.constrain();
    sc.clock_setup.oscillator = get_oscillator();
    let clocks = sc.clock_setup.freeze();

    #[cfg(feature = "tm4c1294")]
    let mut porta = p.GPIO_PORTA_AHB.split(&sc.power_control);
    #[cfg(feature = "tm4c123")]
    let mut porta = p.GPIO_PORTA.split(&sc.power_control);

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
        115_200_u32.bps(),
        hal::serial::NewlineMode::SwapLFtoCRLF,
        &clocks,
        &sc.power_control,
    );

    fe_rtos::arch::arch_setup(&mut cp);

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

    fe_osi::task::task_spawn(fe_rtos::task::DEFAULT_STACK_SIZE, stdio::stdout, None);
    fe_osi::task::task_spawn(fe_rtos::task::DEFAULT_STACK_SIZE, stdio::stdin, None);
    fe_osi::task::task_spawn(fe_rtos::task::DEFAULT_STACK_SIZE, cmd::shell, None);

    //It's probably a good idea to have the context switch be the lowest
    //priority interrupt.
    unsafe {
        cp.SCB
            .set_priority(cortex_m::peripheral::scb::SystemHandler::PendSV, 7);
    }

    //Start the FeRTOS scheduler
    let enable_systick = |reload: usize| {
        cp.SYST.set_reload(reload as u32);
        cp.SYST.clear_current();
        cp.SYST.enable_counter();
        cp.SYST.enable_interrupt();
    };
    let reload_val = cortex_m::peripheral::SYST::get_ticks_per_10ms() / 10;
    fe_rtos::task::start_scheduler(enable_systick, reload_val as usize);

    loop {}
}
