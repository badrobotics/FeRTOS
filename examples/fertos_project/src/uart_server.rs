use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;
use embedded_hal::serial::{Read as SerialRead, Write as SerialWrite};

pub fn uart_transmit_server<T: SerialWrite<u8> + Write>(serial: &mut T) {
    let mut subscriber = fe_osi::ipc::Subscriber::new("uart_tx").unwrap();
    loop {
        if let Some(message) = subscriber.get_message() {
            let s = String::from_utf8_lossy(&message);
            write!(serial, "{}", s).unwrap();
        }
    }
}

pub fn uart_receive_server<T: SerialRead<u8>>(serial: &mut T) {
    let mut publisher = fe_osi::ipc::Publisher::new("uart_rx").unwrap();

    loop {
        match serial.read() {
            Ok(c) => {
                let mut v = Vec::new();
                v.push(c);
                publisher.publish(v).unwrap();
            }
            Err(_) => {
                fe_osi::sleep(1);
            }
        };
    }
}
