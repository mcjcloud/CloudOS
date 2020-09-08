#![no_std]  // exclude the standard library (which will not exist by default in our OS)
#![no_main] // because the C runtime isn't being used, a main function cannot be called

// This library is important to the linker but normally comes from libc
extern crate rlibc;

use core::panic::PanicInfo;

/// This function is called on panic. It is needed here because the std implementation is excluded
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// String type doesn't exist since there's no std :( so we use a byte array
static HELLO: &[u8] = b"Hello World!";

// no_mangle prevents this function name from being changed by the compiler.
// the linker will specifically look for a function named _start so this is important
#[no_mangle]
// 'extern "C"' tells the Rust compiler to use the C calling convention for this function
pub extern "C" fn _start() -> ! {
    // create a pointer to the address at which the VGA buffer lives
    let vga_buffer = 0xb8000 as *mut u8;

    // iterate over each character and write it to the VGA buffer
    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            // i is the number of characters to offset by
            // multiply by 2 because each character on-screen requires two bytes (the character and the color)
            *vga_buffer.offset(i as isize * 2) = byte;      // set the character value
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;   // the color value (cyan)
        }
    }

    // never return
    loop {}
}
