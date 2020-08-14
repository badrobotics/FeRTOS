extern crate alloc;

use crate::spinlock::Spinlock;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use fe_osi::allocator::LayoutFFI;

/******************************************************************************
* The structure of a heap is the following:
*    ---------------------------------------------
*   | free list |         usable heap            |
*    ---------------------------------------------
*   The beginning of the heap is reserved for the free list.
*   The ith bit of the free list is 0 if the ith block in the usable heap is
*   unallocated and is 1 otherwise.
*   The "usable heap" is the part of the heap that will be used to service
*   dynamic memory allocations.
*
*   In the code below, the usable heap will simple be refered to as the "heap."
******************************************************************************/

const BLOCK_SIZE: usize = 0x10;
static ALLOC_LOCK: Spinlock = Spinlock::new();
static mut FREE_LIST: *mut u8 = null_mut();
static mut NUM_BLOCKS: usize = 0;
static mut HEAP: *mut u8 = null_mut();
static mut HEAP_REMAINING: usize = 0;

/// The global allocator for FeRTOS.
/// KernelAllocator is enabled automatically by the crate
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

// Returns the appropriate bit mask for a bit position with a certain number of
// bits remaining to process. Also updates cur_block by the number of blocks/bits
// included in the returned bit mask.
fn get_bit_mask(bit: usize, blocks_remaining: usize, cur_block: &mut usize) -> u8 {
    // Look up table for potential bit masks
    const BIT_LUT: [u8; 8] = [0x01, 0x03, 0x07, 0x0F, 0x1F, 0x3F, 0x7F, 0xFF];

    //If we need to include more bits than are represented in
    //this byte of the free list, set the rest of the bits,
    //otherwise, just set the bits you need.
    if blocks_remaining >= 8 - bit {
        *cur_block += 8 - bit;
        BIT_LUT[8 - bit - 1] << bit
    } else {
        *cur_block += blocks_remaining;
        BIT_LUT[blocks_remaining - 1] << bit
    }
}

pub(crate) unsafe fn alloc(layout: LayoutFFI) -> *mut u8 {
    //If HEAP points to zero, we need to initialize the heap
    ALLOC_LOCK.take();
    if (HEAP as usize) == 0 {
        init_heap();
    }
    ALLOC_LOCK.give();

    let mut data_ptr = null_mut();
    let mut cur_block = 0;
    let blocks_needed = if layout.size % BLOCK_SIZE == 0 {
        layout.size / BLOCK_SIZE
    } else {
        (layout.size / BLOCK_SIZE) + 1
    };

    ALLOC_LOCK.take();

    //Loop through the heap to find an unallocated group of blocks with enough room
    while cur_block + blocks_needed < NUM_BLOCKS {
        //Determine the location corresponding to this block
        let ptr = HEAP.add(cur_block * BLOCK_SIZE);

        //If this block doesn't meet the alignment requirements, move on
        if (ptr as usize) & (layout.align - 1) != 0 {
            cur_block += 1;
            continue;
        }

        //See if there's enough room at this block
        let start_block = cur_block;
        data_ptr = ptr;
        while cur_block < start_block + blocks_needed {
            let blocks_remaining = blocks_needed - (cur_block - start_block);
            let idx = cur_block / 8;
            let bit = cur_block % 8;
            let bit_mask = get_bit_mask(bit, blocks_remaining, &mut cur_block);

            if *FREE_LIST.add(idx) & bit_mask != 0 {
                data_ptr = null_mut();
                break;
            }
        }

        if !data_ptr.is_null() {
            //Mark the data locations as allocated
            cur_block = start_block;
            while cur_block < start_block + blocks_needed {
                let blocks_remaining = blocks_needed - (cur_block - start_block);
                let idx = cur_block / 8;
                let bit = cur_block % 8;
                let bit_mask = get_bit_mask(bit, blocks_remaining, &mut cur_block);

                *FREE_LIST.add(idx) |= bit_mask;
            }
            HEAP_REMAINING -= blocks_needed * BLOCK_SIZE;
            break;
        }
    }

    ALLOC_LOCK.give();

    data_ptr
}

pub(crate) unsafe fn dealloc(ptr: *mut u8, layout: LayoutFFI) {
    //Determine the block that the pointer belongs to
    let start_block = (ptr as usize - HEAP as usize) / BLOCK_SIZE;
    let mut cur_block = start_block;

    ALLOC_LOCK.take();
    let num_blocks = if layout.size % BLOCK_SIZE == 0 {
        layout.size / BLOCK_SIZE
    } else {
        (layout.size / BLOCK_SIZE) + 1
    };

    while cur_block < start_block + num_blocks {
        let blocks_remaining = num_blocks - (cur_block - start_block);
        let idx = cur_block / 8;
        let bit = cur_block % 8;
        let bit_mask = !get_bit_mask(bit, blocks_remaining, &mut cur_block);

        *FREE_LIST.add(idx) &= bit_mask;
    }

    HEAP_REMAINING += num_blocks * BLOCK_SIZE;

    ALLOC_LOCK.give();
}

unsafe fn init_heap() {
    //First determine where the heap is
    extern "C" {
        static mut _sheap: u8;
        static mut _eheap: u8;
    }
    //Make sure FREE_LIST and total_heap_size are block aligned. It just makes things easier
    FREE_LIST =
        (((&mut _sheap as *mut u8 as usize) + (BLOCK_SIZE - 1)) & !(BLOCK_SIZE - 1)) as *mut u8;
    let total_heap_size =
        ((&mut _eheap as *mut u8 as usize) - (FREE_LIST as usize)) & !(BLOCK_SIZE - 1);
    // Determining the size of the heap and the free list is a little tricky.
    // First, for brevity, let's define some variables:
    // x is the size of the free list, y is the size of the heap, z is the total
    // heap size, and b is the block size.
    // We know simply that z = x + y
    // We also know that each bit of the free list represents a block in the
    // heap, and there are 8 bits in a byte so:
    // x = y / (8b)
    // From substitution with the equation for z, we get
    // z = (y / (8b)) + y
    // Which we can simplify to
    // y = z * (8b)/(8b + 1)
    // We will also do some math to make sure that y % b == 0
    // We are also assuming the entire heap and heap size is block size aligned
    let usable_heap_size =
        (total_heap_size * (8 * BLOCK_SIZE) / ((8 * BLOCK_SIZE) + 1)) & !(BLOCK_SIZE - 1);
    NUM_BLOCKS = usable_heap_size / BLOCK_SIZE;
    let free_list_size = total_heap_size - usable_heap_size;
    HEAP = FREE_LIST.add(free_list_size);

    HEAP_REMAINING = usable_heap_size;

    // Set all of the blocks to free
    core::ptr::write_bytes(FREE_LIST, 0, free_list_size);
}

pub fn get_heap_remaining() -> usize {
    unsafe { HEAP_REMAINING }
}
