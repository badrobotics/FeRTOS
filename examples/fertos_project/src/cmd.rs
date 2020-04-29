extern crate alloc;
use alloc::string::String;
use fe_osi;

pub fn cmd(_: &mut u8) {
    let mut stdin = fe_osi::ipc::Subscriber::new("stdin");
    let mut stdout = fe_osi::ipc::Publisher::new("stdout");
    loop {
        let message = stdin.get_message();
        stdout.publish(message);
    }
}

pub fn hello_world(_: &mut u8) {
    let mut stdout = fe_osi::ipc::Publisher::new("stdout");
    loop {
        let msg = String::from("Hello, World!\r\n").into_bytes();
        stdout.publish(msg);
        fe_osi::sleep(100);
    }
}