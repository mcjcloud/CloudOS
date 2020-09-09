#![no_std] // exclude the standard library (which will not exist by default in our OS)
#![no_main] // because the C runtime isn't being used, a main function cannot be called
#![feature(custom_test_frameworks)] // enable creating a custom test framework since the test crate is in std
#![test_runner(cloudos::test_runner)] // specifies the test_runner function as the test runner
#![reexport_test_harness_main = "test_main"] // export the test runner function with test_main name

// This library is important to the linker but normally comes from libc
extern crate rlibc;

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

// no_mangle prevents this function name from being changed by the compiler.
// the linker will specifically look for a function named _start so this is important
#[no_mangle]
// 'extern "C"' tells the Rust compiler to use the C calling convention for this function
pub extern "C" fn _start() -> ! {
  println!("Hello World{}", "!");

  cloudos::init();

  #[cfg(test)]
  test_main();

  println!("Didn't crash!");

  // never return
  cloudos::hlt_loop();
}
