use core::alloc::{GlobalAlloc, Layout};

pub struct FeAllocator;

#[repr(C)]
pub struct LayoutFFI {
    pub size: usize,
    pub align: usize
}

extern "C" {
    fn do_alloc(layout: LayoutFFI) -> *mut u8;
    fn do_dealloc(ptr: *mut u8, layout: LayoutFFI) -> usize;
}

unsafe impl GlobalAlloc for FeAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let layout_ffi: LayoutFFI = LayoutFFI {
            size: layout.size(),
            align: layout.align()
        };
        do_alloc(layout_ffi)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let layout_ffi: LayoutFFI = LayoutFFI {
            size: layout.size(),
            align: layout.align()
        };
        do_dealloc(ptr, layout_ffi);
    }
}

#[alloc_error_handler]
fn error_handler(_: Layout) -> ! {
    loop {}
}
