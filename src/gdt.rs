use lazy_static::lazy_static;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use x86_64::VirtAddr;

// initialize the Global Descriptor Table
// the GDT is a table for memory segmentation
// it will allow us to switch stacks on a double fault, avoiding a triple fault
pub fn init() {
  use x86_64::instructions::segmentation::set_cs;
  use x86_64::instructions::tables::load_tss;
  GDT.0.load();
  // Tell the CPU to use the loaded code selector and tss selector
  // this is unsafe because it's possible to load invalid selectors
  unsafe {
    set_cs(GDT.1.code_selector);
    load_tss(GDT.1.tss_selector);
  }
}

// describes where in the IST the stack pointer goes
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

// lazily initialize the Task State Segment (TSS)
// TSS holds two stack tables
lazy_static! {
  static ref TSS: TaskStateSegment = {
    let mut tss = TaskStateSegment::new();
    // write to the 0th IST a stack
    // use stack_end because stacks grow from high -> low in x86
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
      const STACK_SIZE: usize = 4096 * 5;
      static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

      let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
      let stack_end = stack_start + STACK_SIZE;
      stack_end
    };
    tss
  };
}

// Selectors is a struct containing the code and tss selector
struct Selectors {
  code_selector: SegmentSelector,
  tss_selector: SegmentSelector,
}

// lazily initialize the GDT
lazy_static! {
  static ref GDT: (GlobalDescriptorTable, Selectors) = {
    let mut gdt = GlobalDescriptorTable::new();
    let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
    let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
    (gdt, Selectors { code_selector, tss_selector })
  };
}
