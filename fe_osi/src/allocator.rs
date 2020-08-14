use crate::{print_msg, putc, usize_to_chars};
use core::alloc::{GlobalAlloc, Layout};

/// The allocator that should be used for standalone FeRTOS binaries.
/// Because FeRTOS does not currently support standalone binaries, this is
/// currently useless.
pub struct FeAllocator;

#[repr(C)]
pub struct LayoutFFI {
    pub size: usize,
    pub align: usize,
}

extern "C" {
    fn do_alloc(layout: LayoutFFI) -> *mut u8;
    fn do_dealloc(ptr: *mut u8, layout: LayoutFFI) -> usize;
    fn do_get_heap_remaining() -> usize;
}

/// Returns the total amount of free memory in the heap.
/// Note: This is simply a debugging tool. Because it does not consider
/// fragmentation inside the heap, it cannot indicate whether or not an allocation
/// will succeed.
pub fn get_heap_remaining() -> usize {
    unsafe { do_get_heap_remaining() }
}

unsafe impl GlobalAlloc for FeAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let layout_ffi: LayoutFFI = LayoutFFI {
            size: layout.size(),
            align: layout.align(),
        };
        do_alloc(layout_ffi)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let layout_ffi: LayoutFFI = LayoutFFI {
            size: layout.size(),
            align: layout.align(),
        };
        do_dealloc(ptr, layout_ffi);
    }
}

#[alloc_error_handler]
fn error_handler(layout: Layout) -> ! {
    //You should only need 20 digits to represent a 64-bit number
    let mut digits: [u8; 20] = [0; 20];

    // We don't use panic! here because the panic handler does heap allocation
    print_msg("Out of memory!\n");
    print_msg("Size: ");
    let start_idx = usize_to_chars(&mut digits, layout.size());
    print_msg(core::str::from_utf8(&digits[start_idx..]).unwrap());
    print_msg(" Alignment: ");
    let start_idx = usize_to_chars(&mut digits, layout.align());
    print_msg(core::str::from_utf8(&digits[start_idx..]).unwrap());
    putc('\n');

    unsafe {
        crate::exit();
    }

    loop {}
}
