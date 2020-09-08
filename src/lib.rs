// The purpose of this file is to make certain functionality available to both integration tests and source code
#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)] // enable custom test frameworks
#![feature(abi_x86_interrupt)]      // enable "x86-interrupt" calling convention
#![test_runner(crate::test_runner)] // use test_runner for tests
#![reexport_test_harness_main = "test_main"]

extern crate rlibc;

// make modules available to crate
pub mod interrupts;
pub mod gdt;
pub mod serial;
pub mod vga_buffer;

use core::panic::PanicInfo;

pub fn init() {
  gdt::init();
  interrupts::init_idt();
}

pub trait Testable {
  fn run(&self) -> ();
}

// Testable trait adds a run function to all functions with Fn() trait
impl<T> Testable for T
where
  T: Fn(),
{
  fn run(&self) {
    serial_print!("{}...\t", core::any::type_name::<T>());
    self();
    serial_println!("[ok]");
  }
}

/**
 * test_runner runs all functions with the Testable trait
 */
pub fn test_runner(tests: &[&dyn Testable]) {
  serial_println!("Running {} tests", tests.len());
  for test in tests {
    test.run();
  }
  exit_qemu(QemuExitCode::Success);
}

/**
 * test_panic_handler gracefully handles panics and exits QEMU
 */
pub fn test_panic_handler(info: &PanicInfo) -> ! {
  serial_println!("[failed]\n");
  serial_println!("Error: {}\n", info);
  exit_qemu(QemuExitCode::Failed);
  loop {}
}

/// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
  test_main();
  loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  test_panic_handler(info)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
  Success = 0x10,
  Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
  use x86_64::instructions::port::Port;

  unsafe {
    let mut port = Port::new(0xf4);
    port.write(exit_code as u32);
  }
}
