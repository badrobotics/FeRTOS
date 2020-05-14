extern crate alloc;
use fe_osi;

//pub fn cmd(_: &mut u8) {
//    let mut stdin = fe_osi::ipc::Subscriber::new("stdin").unwrap();
//    let mut stdout = fe_osi::ipc::Publisher::new("stdout");
//    loop {
//        if let Some(message) = stdin.get_message() {
//            stdout.publish(message);
//        }
//    }
//}

pub fn hello_world(_: &mut u8) {
    let mut stdout = fe_osi::ipc::Publisher::new("stdout");
    let mut count: usize = 0;
    loop {
        let msg = format!("Hello, World! {}\r\n", count).into_bytes();
        stdout.publish(msg).unwrap();
        count += 1;
    }
}