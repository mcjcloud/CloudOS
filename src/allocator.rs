use alloc::alloc::{GlobalAlloc, Layout};
use bump::BumpAllocator;
use core::ptr::null_mut;
use linked_list_allocator::LockedHeap;
use x86_64::{
  structures::paging::{
    mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
  },
  VirtAddr,
};

pub mod bump;

#[global_allocator]
static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());
// static ALLOCATOR: LockedHeap = LockedHeap::empty();
// static ALLOCATOR: Dummy = Dummy;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

/**
 * init_heap maps a range of virtual address to physical addresses to be used for the heap
 */
pub fn init_heap(
  mapper: &mut impl Mapper<Size4KiB>,
  frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
  // create page range for heap
  let page_range = {
    let heap_start = VirtAddr::new(HEAP_START as u64); // virt addr for heap start
    let heap_end = heap_start + HEAP_SIZE - 1u64; // virt addr for heap end
    let heap_start_page = Page::containing_address(heap_start); // create page for start
    let heap_end_page = Page::containing_address(heap_end); // create page for end
    Page::range_inclusive(heap_start_page, heap_end_page) // create page range
  };

  // allocate pages to physical frames
  for page in page_range {
    let frame = frame_allocator
      .allocate_frame()
      .ok_or(MapToError::FrameAllocationFailed)?;
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
  }

  // init the allocator with the heap addresses
  unsafe {
    ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
  }

  Ok(())
}

/**
 * align addr upwards to alignment align
 * if addr is not a multiple of the alignment, make it so
 */
fn align_up(addr: usize, align: usize) -> usize {
  // verbose implementation
  // let remainder = addr % align;
  // if remainder == 0 {
  //   addr
  // } else {
  //   addr - remainder + align
  // }

  // because align will be a power of 2 (bc of GlobalAlloc trait), it has only one bit set
  // so align - 1 must have all lower bits set (e.g. 0b00100000 - 1 = 0b00011111)
  // NOTing this gives us all bits not lower than align
  // the & on the address aligns the bits downward
  // we add align - 1 first to align upward
  (addr + align - 1) & !(align - 1)
}

pub struct Dummy;

unsafe impl GlobalAlloc for Dummy {
  unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
    null_mut()
  }

  unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
    panic!("dalloc should never be called")
  }
}

// A wrapper around spin::Mutex allowing trait implementations
pub struct Locked<A> {
  inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
  pub const fn new(inner: A) -> Self {
    Locked {
      inner: spin::Mutex::new(inner),
    }
  }

  pub fn lock(&self) -> spin::MutexGuard<A> {
    self.inner.lock()
  }
}
