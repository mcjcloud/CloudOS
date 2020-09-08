use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

// Color represents the 16 color options
#[allow(dead_code)] // prevent warnings for unused colors
#[derive(Debug, Clone, Copy, PartialEq, Eq)] // use basic implementation for given traits
#[repr(u8)] // each value below will default to u8 size
pub enum Color {
  Black = 0,
  Blue = 1,
  Green = 2,
  Cyan = 3,
  Red = 4,
  Magenta = 5,
  Brown = 6,
  LightGray = 7,
  DarkGray = 8,
  LightBlue = 9,
  LightGreen = 10,
  LightCyan = 11,
  LightRed = 12,
  Pink = 13,
  Yellow = 14,
  White = 15,
}

// ColorCode is a tuple struct representing a Color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] // ensures that ColorCode has the same data layout as u8
struct ColorCode(u8);

impl ColorCode {
  fn new(foreground: Color, background: Color) -> ColorCode {
    // create a byte with the bg as the first 4 bits and fg as the last 4
    ColorCode((background as u8) << 4 | (foreground as u8))
  }
}

// ScreenChar is a struct representing a character and its color on screen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // do what C does
struct ScreenChar {
  ascii_character: u8,
  color_code: ColorCode,
}

// screen is 80x25 spaces
const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

// Buffer represents the VGA screenspace
#[repr(transparent)]
struct Buffer {
  // reminder: array definition is [type: size]
  // so this is a 2D array of size BUFFER_WIDTHxBUFFER_HEIGHT
  chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// Writer keeps track of the cursor and a reference to the screen buffer
pub struct Writer {
  column_position: usize,
  color_code: ColorCode,
  buffer: &'static mut Buffer,
}

impl Writer {
  /**
   * write a byte to VGA address space
   */
  pub fn write_byte(&mut self, byte: u8) {
    match byte {
      b'\n' => self.new_line(), // if the byte is a newline, create a new line
      byte => {
        // if the column is at the end of the screen, create a new line
        if self.column_position >= BUFFER_WIDTH {
          self.new_line();
        }

        let row = BUFFER_HEIGHT - 1; // the bottom row
        let col = self.column_position; // the current column position

        // create a screenchar at the given location in the array
        self.buffer.chars[row][col].write(ScreenChar {
          ascii_character: byte,
          color_code: self.color_code,
        });
        // increment the column position
        self.column_position += 1;
      }
    }
  }

  /**
   * write a string to the screen
   */
  pub fn write_string(&mut self, s: &str) {
    for byte in s.bytes() {
      match byte {
        0x20..=0x7e | b'\n' => self.write_byte(byte), // printable ascii
        _ => self.write_byte(0xfe),                   // not printable, print a square
      }
    }
  }

  /**
   * overwrite the entire screen with spaces
   */
  pub fn clear_screen(&mut self) {
    for row in 0..BUFFER_HEIGHT {
      self.clear_row(row);
    }
  }

  /**
   * create a new line, pushing all other lines up
   */
  fn new_line(&mut self) {
    for row in 1..BUFFER_HEIGHT {
      for col in 0..BUFFER_WIDTH {
        let character = self.buffer.chars[row][col].read();
        self.buffer.chars[row - 1][col].write(character);
      }
    }
    self.clear_row(BUFFER_HEIGHT - 1);
    self.column_position = 0;
  }

  /**
   * overwrite the given row with spaces
   */
  fn clear_row(&mut self, row: usize) {
    let blank = ScreenChar {
      ascii_character: b' ',
      color_code: self.color_code,
    };
    for col in 0..BUFFER_WIDTH {
      self.buffer.chars[row][col].write(blank);
    }
  }
}

// implement the Write trait to allow the println! macro to be used
impl fmt::Write for Writer {
  fn write_str(&mut self, s: &str) -> fmt::Result {
    self.write_string(s);
    return Ok(());
  }
}

// create a lazily initialized static writer
// this is necessary because references to pointers cannot be determined at compile-time
lazy_static! {
  // the use of spin Mutex allows safe access to the writer without the concept of threads
  pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
    column_position: 0,
    color_code: ColorCode::new(Color::Yellow, Color::Black),
    buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
  });
}

// Define macros to allow easy printing

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! clear_screen {
  () => {
    $crate::vga_buffer::_clear_screen()
  };
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
  use core::fmt::Write;
  WRITER.lock().write_fmt(args).unwrap();
}

#[doc(hidden)]
pub fn _clear_screen() {
  use core::fmt::Write;
  WRITER.lock().clear_screen();
}

#[test_case]
fn test_println_simple() {
  println!("test println simple");
}

#[test_case]
fn test_println_many() {
  for _ in 0..200 {
    println!("test println many output");
  }
}

#[test_case]
fn test_println_output() {
  let s = "Some test string";
  println!("{}", s);
  for (i, c) in s.chars().enumerate() {
    let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
    assert_eq!(char::from(screen_char.ascii_character), c);
  }
}

#[test_case]
fn test_clear_screen() {
  clear_screen!();
}
