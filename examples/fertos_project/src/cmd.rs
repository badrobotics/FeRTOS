extern crate alloc;
use crossbeam_queue::PopError;
use alloc::vec::Vec;
use alloc::string::String;

pub fn cmd(_: &mut u8) {
    let mut stdin = fe_osi::ipc::Subscriber::new("stdin");
    let mut stdout = fe_osi::ipc::Publisher::new("stdout");
    loop {

        let message = stdin.get_message();
        stdout.publish(message);
    }
}