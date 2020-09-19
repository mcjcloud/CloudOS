#![no_std] // exclude the standard library (which will not exist by default in our OS)
#![no_main] // because the C runtime isn't being used, a main function cannot be called
#![feature(custom_test_frameworks)] // enable creating a custom test framework since the test crate is in std
#![test_runner(cloudos::test_runner)] // specifies the test_runner function as the test runner
#![reexport_test_harness_main = "test_main"] // export the test runner function with test_main name

// This library is important to the linker but normally comes from libc
extern crate alloc;
extern crate rlibc;

use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};
use bootloader::{entry_point, BootInfo};
use cloudos::allocator;
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
  use x86_64::VirtAddr;

  println!("Hello World{}", "!");

  cloudos::init();

  // grab reference to l4 table in virt memory
  let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
  let mut mapper = unsafe { memory::init(phys_mem_offset) };
  let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

  allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap init failed");

  // allocate a number on the heap
  let heap_value = Box::new(41);
  println!("heap_value at {:p}", heap_value);

  // create dynamically sized vector
  let mut vec = Vec::new();
  for i in 0..500 {
    vec.push(i);
  }
  println!("vec at {:p}", vec.as_slice());

  // create ref counted vecotr -> will be freed when count reaches 0
  let reference_counted = Rc::new(vec![1, 2, 3]);
  let cloned_reference = reference_counted.clone();
  println!("current reference count is {}", Rc::strong_count(&cloned_reference));
  core::mem::drop(reference_counted);
  println!("reference count is {} now", Rc::strong_count(&cloned_reference));

  #[cfg(test)]
  test_main();

  println!("Didn't crash!");

  // never return
  cloudos::hlt_loop();
}
