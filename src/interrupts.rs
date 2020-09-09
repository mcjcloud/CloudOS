//  PIC DIAGRAM         ____________                          ____________
// Real Time Clock --> |            |   Timer -------------> |            |
// ACPI -------------> |            |   Keyboard-----------> |            |      _____
// Available --------> | Secondary  |----------------------> | Primary    |     |     |
// Available --------> | Interrupt  |   Serial Port 2 -----> | Interrupt  |---> | CPU |
// Mouse ------------> | Controller |   Serial Port 1 -----> | Controller |     |_____|
// Co-Processor -----> |            |   Parallel Port 2/3 -> |            |
// Primary ATA ------> |            |   Floppy disk -------> |            |
// Secondary ATA ----> |____________|   Parallel Port 1----> |____________|

use crate::gdt;
use crate::print;
use crate::println;
use lazy_static::lazy_static;
use pic8259_simple::ChainedPics;
use spin;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub const PIC_1_OFFSET: u8 = 32; // Interrupt Controller should start at port 32 (first free after 32 fault ports)
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8; // second controller goes after the first

// PICS represents the diagram above, made read/write safe by a Mutex
// this is unsafe because PIC_1_OFFSET and PIC_2_OFFSET could be invalid
pub static PICS: spin::Mutex<ChainedPics> =
  spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

// InterruptIndex represents the index of the interrupts in the diagram above
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
  Timer = PIC_1_OFFSET,
  Keyboard,
}

impl InterruptIndex {
  fn as_u8(self) -> u8 {
    self as u8
  }

  fn as_usize(self) -> usize {
    usize::from(self.as_u8())
  }
}

// lazily initialize the IDT
// the Interrupt Descriptor Table (IDT) maps interrupt codes to
// their corresponding handler
lazy_static! {
  static ref IDT: InterruptDescriptorTable = {
    let mut idt = InterruptDescriptorTable::new();

    // fault interrupts
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    unsafe {
      idt
        .double_fault
        .set_handler_fn(double_fault_handler)
        .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    }

    // PIC interrupts
    idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
    idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);

    // evaluate to the idt
    idt
  };
}

pub fn init_idt() {
  IDT.load();
}

/**
 * breakpoint_handler handles breakpoint interrupts
 */
extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
  println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

/**
 * double_fault_handler handles a double fault
 */
extern "x86-interrupt" fn double_fault_handler(
  stack_frame: &mut InterruptStackFrame,
  _error_code: u64,
) -> ! {
  panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

/**
 * timer_interrupt_handler handles interrupt from the timer in the PIC
 */
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
  print!(".");

  // send "end of interrupt"
  unsafe {
    PICS
      .lock()
      .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
  }
}

/**
 * keyboard_interrupt_handler handles keystrokes
 */
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
  use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
  use spin::Mutex;
  use x86_64::instructions::port::Port;

  // define static keyboard
  lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
      Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
    );
  }

  let mut keyboard = KEYBOARD.lock();
  let mut port = Port::new(0x60); // data port for PS/2 controller

  // read scancode, if it is a valid value, print it
  let scancode: u8 = unsafe { port.read() };
  if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
    if let Some(key) = keyboard.process_keyevent(key_event) {
      match key {
        DecodedKey::Unicode(character) => print!("{}", character),
        DecodedKey::RawKey(key) => print!("{:?}", key),
      }
    }
  }

  // notify end of interrupt
  unsafe {
    PICS
      .lock()
      .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
  }
}

// #[test_case]
// fn test_breakpoint_exception() {
//   x86_64::instructions::interrupts::int3();
// }
