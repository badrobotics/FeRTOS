//extern crate alloc;
//
//pub fn cmd(_: &mut u8) {
//    let mut stdin = fe_osi::ipc::Subscriber::new("stdin");
//    let mut stdout = fe_osi::ipc::Publisher::new("stdout");
//    loop {
//
//        let message = stdin.get_message();
//        stdout.publish(message);
//    }
//}