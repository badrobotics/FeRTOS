use embedded_hal::serial::{Read as SerialRead, Write as SerialWrite};
use core::fmt::Write;
use alloc::string::String;
use alloc::vec::Vec;

//use crate::stdio::{STDIN, STDOUT};


pub fn uart_transmit_server<T: SerialWrite<u8> + Write>(serial: &mut T) {
    let mut subscriber = fe_osi::ipc::Subscriber::new("uart_test");
    loop {
        let message = subscriber.get_message();
        let s = String::from_utf8_lossy(&message);
        write!(serial, "{}", s).unwrap();
    }
}

pub fn uart_receive_server<T: SerialRead<u8>>(serial: &mut T) {
    //let subscriber = fe_osi::ipc::Subscriber::new("uart_receiver");
    let mut publisher = fe_osi::ipc::Publisher::new("uart_test");

    loop {
        match serial.read() {
            Ok(c) => {
                let mut v = Vec::new();
                v.push(c);
                publisher.publish(v);
            }
            Err(_) => {
                fe_osi::sleep(10);
            }
        };
    }
}