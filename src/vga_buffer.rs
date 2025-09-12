use volatile::Volatile;         // Import the Volatile type to prevent compiler optimizations for VGA buffer writes
use core::fmt;                  // Support Rust's formatting macros to easily print different types
use lazy_static::lazy_static;   // For initializing static Writer at runtime
use spin::Mutex;                // Add safe interior mutability for static Writer

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(
        Writer {
            column_position: 0,
            color_code: ColorCode::new(Color::Yellow, Color::Black),
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },   // VGA text buffer memory address
        }
    );
}

#[allow(dead_code)]                             // We don't use all the colors, so we disable the warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]    // These traits enable copy semantics to make it printable and comparable.
#[repr(u8)]                                     // Ensure the enum is represented as a u8 in memory (only 4 bits are needed however).
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]                            // Ensure the struct has the same memory layout as its single field
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]                                       // Ensure the struct fields are laid out in memory in the order they are defined
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_string(&mut self, string: &str) {
        for byte in string.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),   // Printable ASCII byte or newline
                _ => self.write_byte(0xfe),                     // Not part of printable ASCII range => print ■
            }
        }
    }

    /// Write a byte to the VGA buffer. Writes to the last row, inserting a newline if necessary.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                });

                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        // Shift all rows up by one
        for row in 1..BUFFER_HEIGHT {   // Row 0 is shifted off the screen
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

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

impl fmt::Write for Writer {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        self.write_string(string);
        Ok(())                                              // Return OK result containing the () type
    }
}

// Function needs to be public so it can be accessed from the print! macro, but is hidden from documentation.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

// Macro for print functionality (modified from standard library macro)
// Note: macro_export places macro at the crate root, making it accessible from other modules.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

// Macro for println functionality (modified from standard library macro)
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}