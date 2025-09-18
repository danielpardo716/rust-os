// Bump allocator implementation
// A bump allocator allocates memory by moving a "next" pointer forward.
// The allocation counter increases by 1 for each allocation, and decreases by 1 for each deallocation.
// When the counter reaches zero, the allocator can be reset to free all allocations at once.
// It is very fast and simple, but it does not support deallocation of individual allocations.
// NOTE: This will fail heap_allocation::many_boxes_long_lived as it deallocates once, but tries to allocate again while count is 1.

use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;

/// A simple bump allocator.
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    /// Creates a new empty bump allocator.
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    /// Initializes the bump allocator with the given heap bounds.
    ///
    /// This method is unsafe because the caller must ensure that the given
    /// memory range is unused. Also, this method must be called only once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock();               // Lock the mutex to get mutable access
        
        let alloc_start = align_up(bump.next, layout.align());          // Start allocation at next pointer aligned to layout
        let alloc_end = match alloc_start.checked_add(layout.size()) {    // Calculate end of allocation
            Some(end) => end,
            None => return core::ptr::null_mut(),                                // Return null if overflow occurs
        };

        // Only allocate if there is enough space
        if alloc_end > bump.heap_end
        {
            ptr::null_mut()
        }
        else
        {
            bump.next = alloc_end;
            bump.allocations += 1;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut bump = self.lock();
        bump.allocations -= 1;

        // Reset next pointer if no allocations remain
        if bump.allocations == 0 {
            bump.next = bump.heap_start;         
        }    
    }
}