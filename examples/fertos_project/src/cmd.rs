extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use core::str;

pub fn shell(_: &mut u8) {
    let mut stdin = fe_osi::ipc::Subscriber::new("stdin").unwrap();
    let mut stdout = fe_osi::ipc::Publisher::new("stdout").unwrap();

    let prompt = String::from("\r\n>> ");
    let mut cmd_buffer: Vec<u8> = Vec::new();
    match stdout.publish(prompt.clone().into_bytes()) {
        Ok(_) => (),
        Err(_) => panic!("Error publishing"),
    }

    loop {
        let mut msg = stdin.get_message().unwrap();
        match stdout.publish(msg.clone()) {
            Ok(_) => (),
            Err(_) => panic!("Error publishing"),
        }
        cmd_buffer.append(&mut msg);
        match cmd_buffer.iter().position(|&x| x == '\r' as u8) {
            Some(index) => {
                let command: Vec<u8> = cmd_buffer.drain(..index + 1).collect();
                if str::from_utf8(&command).unwrap() == "hello\r" {
                    fe_osi::task::task_spawn(fe_rtos::task::DEFAULT_STACK_SIZE, hello_world, None);
                }

                match stdout.publish(prompt.clone().into_bytes()) {
                    Ok(_) => (),
                    Err(_) => panic!("Error publishing"),
                }
            }
            None => {}
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
