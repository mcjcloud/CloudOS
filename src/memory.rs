// memory.rs is respoinsible for accessing physical memory with virtual mem addresses
// the virtual address space is mapped directly to the physical address space with an offset
// of physical_memory_offset.
//
// reading the physical level 4 table address from the CR3 register and adding the virt addr offset
// gives us the virtual address for the table which the CPU will translate into the physical address
// when we read/write to it.

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
  structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB,
  },
  PhysAddr, VirtAddr,
};

// initialize an OffsetPageTable
// the OffsetPageTable is an x86 crate abstraction for mapping virtual and physical
// memory and assumes that the virt address space is completely mapped to the physical
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
  let level_4_table = active_level_4_table(physical_memory_offset);
  OffsetPageTable::new(level_4_table, physical_memory_offset)
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
  use x86_64::registers::control::Cr3;

  // read the level 4 table frame
  let (level_4_table_frame, _) = Cr3::read();

  let phys = level_4_table_frame.start_address(); // get the physical address start space of the table
  let virt = physical_memory_offset + phys.as_u64(); // calculate the virtual address with the offset
  let page_table_ptr: *mut PageTable = virt.as_mut_ptr(); // create a pointer to the page table in virtual address space

  &mut *page_table_ptr // deref the pointer to create a mutable reference
}

pub struct BootInfoFrameAllocator {
  memory_map: &'static MemoryMap,
  next: usize,
}
impl BootInfoFrameAllocator {
  // create a FrameAllocator from the given memory map
  pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
    BootInfoFrameAllocator {
      memory_map,
      next: 0,
    }
  }

  // create an iterator over the usable frames in the memory map
  // impl Iterator allows us to return some type that implements Iterator without a specifc type
  fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
    // get usable regions of memory
    let regions = self.memory_map.iter();
    let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
    // map each region to its address range
    let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
    // transform to an iterator of frame start addresses
    let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096)); // create an iterator with every 4 KiB item
    // create PhysFrame types from the start addresses
    frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
  }
}
unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
  // use the next availiable frame to allocate
  fn allocate_frame(&mut self) -> Option<PhysFrame> {
    let frame = self.usable_frames().nth(self.next);
    self.next += 1;
    frame
  }
}

/* The x86 mapper abstraction makes the below obsolete but I'm leaving it here anyway for reference
/**
 * provide an unsafe wrapper around the _translate_addr function
 */
pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
  _translate_addr(addr, physical_memory_offset)
}

fn _translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
  use x86_64::registers::control::Cr3;
  use x86_64::structures::paging::page_table::FrameError;

  // read the l4 frame from Cr3 register
  let (level_4_table_frame, _) = Cr3::read();

  // take apart the addr to get the address for each page table
  // table_indices is an array of addresses for each page table
  // if the below methods didn't work, we'd have to use bitwise operators to pull
  // the address apart
  let table_indices = [
    addr.p4_index(),
    addr.p3_index(),
    addr.p2_index(),
    addr.p1_index(),
  ];
  // frame starts as a reference to the l4 table frame
  // but it will be updated below as the table is traversed
  let mut frame = level_4_table_frame;

  // traverse the multi-level page table
  // '&' allows us to borrow each item rather than take ownership
  // index will be a ref to a page table address from l4 -> l1 -> mem
  for &index in &table_indices {
    // convert the frame into a page table ref
    let virt = physical_memory_offset + frame.start_address().as_u64(); // virt address for 'frame' table
    let table_ptr: *const PageTable = virt.as_ptr(); // pointer to 'frame' page table
    let table = unsafe { &*table_ptr }; // reference table

    // read the page table entry in 'frame' and update frame
    let entry = &table[index];
    frame = match entry.frame() {
      Ok(frame) => frame,
      Err(FrameError::FrameNotPresent) => return None,
      Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
    };
  }
  // return the physical address
  Some(frame.start_address() + u64::from(addr.page_offset()))
}
*/
