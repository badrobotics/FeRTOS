extern crate alloc;

pub fn cmd(_: &mut u8) {
    let mut stdin = fe_osi::ipc::Subscriber::new("stdin").unwrap();
    let mut stdout = fe_osi::ipc::Publisher::new("stdout").unwrap();
    loop {
        if let Some(message) = stdin.get_message() {
            stdout.publish(message).unwrap();
        }
    }
}

pub fn hello_world(_: &mut u8) {
    let mut stdout = fe_osi::ipc::Publisher::new("stdout").unwrap();
    let mut count: usize = 0;
    loop {
        let heap_remaining = fe_osi::allocator::get_heap_remaining();
        let msg = format!("Hello, World! {} ({})\r\n", count, heap_remaining).into_bytes();
        stdout.publish(msg).unwrap();
        count += 1;
    }
}

pub fn stdout(_: &mut u8) {
    let mut std_out = fe_osi::ipc::Subscriber::new("stdout").unwrap();
    let mut  uart_tx = fe_osi::ipc::Publisher::new("uart_tx").unwrap();
    loop {
        if let Some(message) = std_out.get_message() {
            uart_tx.publish(message).unwrap();
        }
    }
}

pub fn stdin(_: &mut u8) {
    let mut uart_rx = fe_osi::ipc::Subscriber::new("uart_rx").unwrap();
    let mut  std_in = fe_osi::ipc::Publisher::new("stdin").unwrap();
    loop {
        if let Some(message) = uart_rx.get_message() {
            std_in.publish(message).unwrap();
        }
    }
}