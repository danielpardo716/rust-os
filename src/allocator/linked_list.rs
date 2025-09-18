// Linked list allocator implementation
// A linked list allocator keeps track of free memory regions using a linked list.
// Each free region is represented by a ListNode struct, which contains the size of the region
// and a pointer to the next free region.
// When a memory allocation is requested, the allocator searches the linked list for a suitable
// free region. If a suitable region is found, it is split if necessary, and the allocation is made.
// When memory is deallocated, the region is added back to the linked list, and adjacent free regions are merged.
// NOTE: this implementation does not merge free blocks, causing issues as blocks become fragmented

use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::{mem, ptr};

/// Node for the linked list allocator list.
struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    const fn new(size: usize) -> Self {
        ListNode { size, next: None }
    }

    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

/// A simple linked list allocator.
pub struct LinkedListAllocator {
    head: ListNode,
}

impl LinkedListAllocator {
    /// Creates an empty LinkedListAllocator
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
        }
    }

    /// Initialize the allocator with the given heap bounds.
    ///
    /// This function is unsafe because the caller must guarantee that the given
    /// heap bounds are valid and that the heap is unused. This method must be
    /// called only once.   
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        unsafe {
            self.add_free_region(heap_start, heap_size);
        }
    }

    /// Adds the given memory region to the front of the list.
    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        // Ensure that the freed region is capable of holding ListNode
        assert_eq!(align_up(addr, mem::align_of::<ListNode>()), addr);
        assert!(size >= mem::size_of::<ListNode>());

        // Create a new ListNode and append it to the start of the list
        let mut node = ListNode::new(size);
        node.next = self.head.next.take();                          // Set next to head, reset head to None
        let node_ptr = addr as *mut ListNode;
        unsafe {
            node_ptr.write(node);
            self.head.next = Some(&mut *node_ptr);                  // Set head to node
        }
    }

    /// Looks for a free region with the given size and alignment and removes it from the list.
    /// Returns a tuple of the list node and the start address of the allocation.
    fn find_region(&mut self, size: usize, align: usize) -> Option<(&'static mut ListNode, usize)>
    {
        let mut current = &mut self.head;       // Reference to current list node, updated for each iteration
        
        // Look for a large enough memory region in linked list
        while let Some(ref mut region) = current.next
        {
            if let Ok(alloc_start) = Self::alloc_from_region(&region, size, align)
            {
                // Region suitable for allocation - remove node from list
                let next = region.next.take();
                let ret = Some((current.next.take().unwrap(), alloc_start));
                current.next = next;
                return ret;
            }
            else
            {
                // Region not suitable - continue with next region
                current = current.next.as_mut().unwrap();
            }
        }

        // No suitable region found
        None
    }

    /// Try to use the given region for an allocation with given size and alignment.
    /// Returns the allocation start address on success.
    fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()>
    {
        let alloc_start = align_up(region.start_addr(), align);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;

        if alloc_end > region.end_addr() {
            return Err(());     // Region too small
        }

        let excess_size = region.end_addr() - alloc_end;
        if (excess_size > 0) && (excess_size < mem::size_of::<ListNode>()) {
            return Err(());     // Rest of region too small to hold a ListNode (required because the allocation splits the region in a used and a free part)
        }

        Ok(alloc_start)         // Region suitable for allocation
    }

    /// Adjust the given layout so that the resulting allocated memory region is also capable of storing a ListNode.
    /// Returns the adjusted size and alignment as a (size, align) tuple.
    fn size_align(layout: Layout) -> (usize, usize)
    {
        let layout = layout
            .align_to(mem::align_of::<ListNode>())      // Increase alignment to alignment of ListNode if necessary
            .expect("adjusting alignment failed")
            .pad_to_align();                                                              // Round up size to a multiple of alignment

        let size = layout.size().max(mem::size_of::<ListNode>());
        (size, layout.align())
    }
}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (size, align) = LinkedListAllocator::size_align(layout);
        let mut allocator = self.lock();

        // Find a suitable memory region and remove it from the list
        if let Some((region, alloc_start)) = allocator.find_region(size, align) {
            let alloc_end = alloc_start.checked_add(size).expect("overflow");
            let excess_size = region.end_addr() - alloc_end;
            if excess_size > 0 {
                // Add any excess memory back to the free list
                unsafe {
                    allocator.add_free_region(alloc_end, excess_size);
                }
            }
            alloc_start as *mut u8
        }
        else {
            // No suitable region was found
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let (size, _) = LinkedListAllocator::size_align(layout);
        unsafe {
            self.lock().add_free_region(ptr as usize, size)
        }
    }
}