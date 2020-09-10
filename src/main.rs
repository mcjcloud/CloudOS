#![no_std] // exclude the standard library (which will not exist by default in our OS)
#![no_main] // because the C runtime isn't being used, a main function cannot be called
#![feature(custom_test_frameworks)] // enable creating a custom test framework since the test crate is in std
#![test_runner(cloudos::test_runner)] // specifies the test_runner function as the test runner
#![reexport_test_harness_main = "test_main"] // export the test runner function with test_main name

// This library is important to the linker but normally comes from libc
extern crate rlibc;

use bootloader::{entry_point, BootInfo};
use cloudos::println;
use core::panic::PanicInfo;

// This function is called on panic. It is needed here because the std implementation is excluded
#[cfg(not(test))] // don't use this panic handler in test mode
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  println!("{}", info);
  cloudos::hlt_loop();
}

#[cfg(test)] // use this panic handler in test mode
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  cloudos::test_panic_handler(info);
}

// entry_point macro tells the bootloader the entry point along with the function signature
entry_point!(kernel_main);

// BootInfo is passed from the bootloader to the kernal with info
// this is because of the "map_physical_memory" feature in Cargo.toml
fn kernel_main(boot_info: &'static BootInfo) -> ! {
  use cloudos::memory;
  use x86_64::{structures::paging::Page, VirtAddr};

  println!("Hello World{}", "!");

  cloudos::init();

  // grab reference to l4 table in virt memory
  let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
  let mut mapper = unsafe { memory::init(phys_mem_offset) };
  let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

  // map an unused page
  let page = Page::containing_address(VirtAddr::new(0x0)); // use virt address 0 because we know it is unmapped and requires no new page tables
  // memory::create_example_mapping(page, &mut mapper, &mut frame_allocator); // removed

  // write 'New!' to the screen using the newly mapped page
  let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
  unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

  // let addresses = [
  //   0xb8000,                          // vga buffer
  //   0x201008,                         // some code page
  //   0x0100_00200_1a10,                // some stack page
  //   boot_info.physical_memory_offset, // virt address mapped to physical addres 0x0
  // ];

  // for &address in &addresses {
  //   let virt = VirtAddr::new(address);
  //   let phys = mapper.translate_addr(virt);
  //   println!("{:?} -> {:?}", virt, phys);
  // }

  #[cfg(test)]
  test_main();

  println!("Didn't crash!");

  // never return
  cloudos::hlt_loop();
}
