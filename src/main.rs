#![no_std]  // exclude the standard library (which will not exist by default in our OS)
#![no_main] // because the C runtime isn't being used, a main function cannot be called

// This library is important to the linker but normally comes from libc
extern crate rlibc;

mod vga_buffer;

use core::panic::PanicInfo;

/// This function is called on panic. It is needed here because the std implementation is excluded
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}

// no_mangle prevents this function name from being changed by the compiler.
// the linker will specifically look for a function named _start so this is important
#[no_mangle]
// 'extern "C"' tells the Rust compiler to use the C calling convention for this function
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
    // never return
    loop {}
}
