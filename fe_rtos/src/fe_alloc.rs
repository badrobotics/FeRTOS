use core::alloc::{GlobalAlloc, Layout};
use core::mem::size_of;
use core::ptr::null_mut;
use fe_osi::allocator::LayoutFFI;

/******************************************************************************
* The structure of a heap memory block is the following:
*   block header: size_of(usize) bytes, 2*size_of(usize) byte aligned
*   block data: The data requested by the process
*   padding: padding required to make the next header block 2*size_of(usize) byte aligned
*   block footer: identical to the header
*
* The headers have the following structure:
*   bits 32-3: The size of the data including padding and headers/footers
*   bit 2: 1 if the block is allocated. 0 otherwise
*   bit 1: 1 if the block is the first block. 0 otherwise
*   bit 0: 1 if the block is the last block. 0 otherwise
******************************************************************************/

const HEADER_ALIGN: usize = (2 * size_of::<usize>()) - 1;
const SIZE_MASK: usize = !HEADER_ALIGN;
const ALLOC_MASK: usize = 0x4;
const FIRST_MASK: usize = 0x2;
const LAST_MASK: usize = 0x1;
static mut HEAP: *mut u8 = null_mut();
static mut HEAP_SIZE: usize = 0;
static mut HEAP_REMAINING: usize = 0;

pub struct KernelAllocator;

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let layout_ffi = LayoutFFI {
            size: layout.size(),
            align: layout.align(),
        };

        alloc(layout_ffi)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let layout_ffi = LayoutFFI {
            size: layout.size(),
            align: layout.align(),
        };

        dealloc(ptr, layout_ffi);
    }
}

pub (crate) unsafe fn alloc(layout: LayoutFFI) -> *mut u8 {
    //If HEAP points to, we need to initialize the heap
    if (HEAP as usize) == 0 {
        init_heap();
    }

    let mut header_ptr = HEAP as usize;
    let mut data_ptr = null_mut();
    let heap_max = header_ptr + HEAP_SIZE;
    //The address after the header is usize aligned, so if the  alignment
    //required is greater than that, we need padding before the data pointer.
    //We also need to consider that the header of the next block needs to
    //be 2 * usize aligned.
    let size_needed = {
        //size =               header
        let mut intermediate = size_of::<usize>();
        //           + padding
        intermediate = (intermediate + layout.align - 1) & !(layout.align - 1);
        //           +  data size     + footer
        intermediate += layout.size + size_of::<usize>();
        //           + padding
        intermediate = (intermediate + HEADER_ALIGN) & !HEADER_ALIGN;
        intermediate
    };

    super::disable_interrupts();

    //Loop through the heap to find an unallocated block with enough room
    while header_ptr < heap_max {
        let header_data = *(header_ptr as *const usize);


        if (header_data & ALLOC_MASK) == 0 && (header_data & SIZE_MASK) >= size_needed {
            data_ptr = alloc_block(header_ptr, size_needed, layout);
            break;
        }

        header_ptr += header_data & SIZE_MASK;
    }

    super::enable_interrupts();

    data_ptr
}

pub (crate) unsafe fn dealloc(ptr: *mut u8, layout: LayoutFFI) {
    //Recover the block header from the aligned data pointer
    let header = if layout.align > size_of::<usize>() {
        //If the alignment was big enough, a pointer to the
        //header will be directly behind the data ptr
        *(((ptr as usize) - size_of::<usize>()) as *mut usize)
    } else {
        //Otherwise, the header is directly behind the data pointer
        (ptr as usize) - size_of::<usize>()
    };
    let header_data = *(header as *const usize);
    let old_size = header_data & SIZE_MASK;
    let is_first = (header_data & FIRST_MASK) == FIRST_MASK;
    let is_last = (header_data & LAST_MASK) == LAST_MASK;

    super::disable_interrupts();

    let next_header = header + old_size;
    let next_hdr_data = if is_last {
        //If the current block is the last one, we have no guarentees that
        //reading from the "next" one is valid
        0
    } else {
        *(next_header as *const usize)
    };

    let prev_footer = header - size_of::<usize>();
    let prev_ftr_data = if is_first {
        //If the current block is the first one, we have no guarentees that
        //reading from the "previous" one is valid
        0
    } else {
        *(prev_footer as *const usize)
    };

    //coalesce this block with adjacent free blocks if able
    let left_coalesce = !is_first && (prev_ftr_data & ALLOC_MASK) == 0;
    let right_coalesce = !is_last && (next_hdr_data & ALLOC_MASK) == 0;
    let new_header = if  left_coalesce {
        header - (prev_ftr_data & SIZE_MASK)
    } else {
        header
    };
    let new_footer = if right_coalesce {
        next_header + (next_hdr_data & SIZE_MASK) - size_of::<usize>()
    } else {
        next_header - size_of::<usize>()
    };

    let new_size = new_footer - new_header + size_of::<usize>();

    //Have the header and footer reflect out changes
    //By using the = operator, the ALLOC bit is implicitly set to 0
    *(new_header as *mut usize) = new_size & SIZE_MASK;
    if left_coalesce {
        *(new_header as *mut usize) |= prev_ftr_data & FIRST_MASK;
    } else {
        *(new_header as *mut usize) |= header_data & FIRST_MASK;
    }

    if right_coalesce {
        *(new_header as *mut usize) |= next_hdr_data & LAST_MASK;
    } else {
        *(new_header as *mut usize) |= header_data & LAST_MASK;
    }
    *(new_footer as *mut usize) = *(new_header as *mut usize);

    HEAP_REMAINING += old_size;

    super::enable_interrupts();
}

unsafe fn init_heap() {
    //First determine where the heap is
    extern "C" {
        static mut _sheap: u8;
        static mut _eheap: u8;
    }
    HEAP = &mut _sheap as *mut u8;
    HEAP_SIZE = (&mut _eheap as *mut u8 as usize) - (HEAP as usize);
    HEAP_REMAINING = HEAP_SIZE;

    super::disable_interrupts();

    //Basically just set the initial memory block header and footer
    let heap_ptr = HEAP as usize;
    let header = heap_ptr as *mut usize;
    let footer = (heap_ptr + (HEAP_SIZE & SIZE_MASK) - size_of::<usize>()) as *mut usize;

    *header = (HEAP_SIZE & SIZE_MASK) | FIRST_MASK | LAST_MASK;
    *footer = (HEAP_SIZE & SIZE_MASK) | FIRST_MASK | LAST_MASK;

    super::enable_interrupts();
}

unsafe fn alloc_block(header_ptr: usize, size: usize, layout: LayoutFFI) -> *mut u8{
    let alloc_header = header_ptr as *mut usize;
    let alloc_footer = (header_ptr + size - size_of::<usize>()) as *mut usize;
    let old_size = *alloc_header & SIZE_MASK;
    let unalloc_header = (header_ptr + size) as *mut usize;
    let unalloc_footer = (header_ptr + old_size - size_of::<usize>()) as *mut usize;
    let is_first = *alloc_header & FIRST_MASK;
    let is_last = *alloc_header & LAST_MASK;

    *alloc_header = size | ALLOC_MASK | is_first;
    *alloc_footer = *alloc_header;

    // If the block that we are allocating from is greater than
    // The size that we need, make a new unallocated block with
    // the excess
    if old_size > size {
        *unalloc_header = ((old_size - size) & SIZE_MASK) | is_last;
        *unalloc_footer = *unalloc_header;
    }

    //compute the pointer
    let data_ptr = (header_ptr + size_of::<usize>() + layout.align - 1) & !(layout.align - 1);

    //If the alignment is greater than size_of<usize>(), put a pointer to the
    //header behind the data pointer so we can find it during deallocation
    if layout.align > size_of::<usize>() {
        let ptr = (data_ptr - size_of::<usize>()) as *mut usize;
        *ptr = alloc_header as usize;
    }

    HEAP_REMAINING -= size;

    data_ptr as *mut u8
}

pub fn get_heap_remaining() -> usize {
    unsafe { HEAP_REMAINING }
}
