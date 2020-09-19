use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;

/**
 * represent an allocator using bump method
 */
pub struct BumpAllocator {
  heap_start: usize,  // mem addr of heap start
  heap_end: usize,    // mem addr of heap end
  next: usize,        // pointer to next available page
  allocations: usize, // number of allocated pages
}

impl BumpAllocator {
  /**
   * create new BumpAllocator
   */
  pub const fn new() -> Self {
    BumpAllocator {
      heap_start: 0,
      heap_end: 0,
      next: 0,
      allocations: 0,
    }
  }

  /**
   * initialize a BumpAllocator
   * unsafe because the caller must ensure the heap_start and heap_size are valid
   */
  pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
    self.heap_start = heap_start;
    self.heap_end = heap_start + heap_size;
    self.next = heap_start;
  }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    let mut bump = self.lock(); // get safe reference to self

    // make sure the entire memory region is valid
    // align_up rounds up bump.next to the alignment specified by layout
    // checked_add makes sure the addition does not result in an overflow
    let alloc_start = align_up(bump.next, layout.align());
    let alloc_end = match alloc_start.checked_add(layout.size()) {
      Some(end) => end,
      None => return ptr::null_mut(),
    };
    // make sure the memory isn't going to overflow
    if alloc_end > bump.heap_end {
      ptr::null_mut()
    } else {
      // move next and allocations, return alloc_start as a addr pointer
      bump.next = alloc_end;
      bump.allocations += 1;
      alloc_start as *mut u8
    }
  }

  unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
    let mut bump = self.lock(); // get safe mutable reference

    bump.allocations -= 1; // decrement the allocation count
    if bump.allocations == 0 {
      bump.next = bump.heap_start;
    }
  }
}
