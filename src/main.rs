#![no_std]          // Don't link the Rust standard library
#![no_main]         // Don't use the Rust main function as the entry point
// #![warn(missing_docs)]

use core::panic::PanicInfo;

/// Default Panic handler for the application.
// Since we are using `no_std`, we need to define a panic handler.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {                          // ! is the "never" type, indicating this function will not return
    // This function is called on panic.
    // You can add your own panic handling logic here.
    loop {
        // Infinite loop to halt the program
    }
}

static HELLO: &[u8] = b"Hello, world!\n"; // Static byte string to hold the message

#[unsafe(no_mangle)]                                 // Ensure compiler keeps the name of this function as "_start" as this is the linker-defined entry point
pub extern "C" fn _start() -> ! {                    // Extern "C" - use the C calling convention for starting point instead of Rust calling convention  
    let vga_buffer= 0xb8000 as *mut u8;     // VGA text buffer address in memory, mutable pointer to u8
    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;      // ASCII character byte
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;   // Color byte - light cyan on black background
        }
    }

    loop { }
}