#![no_std]
#![feature(alloc_error_handler)]
#![feature(vec_into_raw_parts)]
#![allow(clippy::empty_loop)]

pub mod allocator;
pub mod interrupt;
pub mod ipc;
pub mod semaphore;
pub mod task;

static mut PUTC: Option<fn(char)> = None;

extern "C" {
    fn do_exit() -> usize;
    fn do_sleep(seconds: u32) -> usize;
    fn do_yield() -> usize;
}

/// Sets what is internally used for putc to print out error messages
pub fn set_putc(putc: fn(char)) {
    unsafe {
        PUTC = Some(putc);
    }
}

/// If a putc function has been set using set_putc, print a character
/// using the putc function.
pub fn putc(c: char) {
    unsafe {
        if let Some(callback) = PUTC {
            callback(c);
        }
    }
}

/// If a putc function has been set using set_putc, print a string
/// using the putc function
pub fn print_msg(msg: &str) {
    for c in msg.chars() {
        putc(c);
    }
}

/// Causes the calling task to exit.
///
/// # Safety
/// Calls to this function may result in a memory leak if the calling task has
/// not freed all of its dynamically allocated memory before hand.
pub unsafe fn exit() -> usize {
    do_exit();
    //Execution should never reach here
    loop {}
}

/// Causes the calling task to block for at least the specified number of
/// milliseconds.
pub fn sleep(ms: u32) -> usize {
    unsafe {
        do_sleep(ms);
    }

    0
}

/// Triggers a context switch.
pub fn r#yield() -> usize {
    unsafe {
        do_yield();
    }

    0
}

/// Puts the ASCII representation of `num` into char_slice.
///
/// # Return
///
/// returns the index of char_slice where the ascii representation begins.
///
/// #Examples
///
/// ```
/// //You need a max of 20 digits to represent a 64 bit number
/// let mut digits: [u8; 20] = [0; 20];
/// let num = 1234;
/// let start_index = usize_to_chars(&mut digits, num);
/// let string: str = core::str::from_utf8(&digits[start_index..]).unwrap();
/// ```
pub fn usize_to_chars(char_slice: &mut [u8], num: usize) -> usize {
    let mut i = char_slice.len() - 1;
    let mut val = num;

    loop {
        //This is some ascii shenanigans
        char_slice[i] = ((val % 10) + 0x30) as u8;
        val /= 10;

        if val == 0 || i == 0 {
            break;
        }

        i -= 1;
    }

    //This is the index where the caller should start printing from
    i
}
