#![no_std]          // Don't link the Rust standard library
#![no_main]         // Don't use the Rust main function as the entry point
// #![warn(missing_docs)]

use core::panic::PanicInfo;

mod vga_buffer;

#[unsafe(no_mangle)]                                 // Ensure compiler keeps the name of this function as "_start" as this is the linker-defined entry point
pub extern "C" fn _start() -> ! {                    // Extern "C" - use the C calling convention for starting point instead of Rust calling convention  
    println!("Hello World{}", "!");
    panic!("Crash and burn");

    loop { }
}

/// Default Panic handler for the application.
// Since we are using `no_std`, we need to define a panic handler.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {                          // ! is the "never" type, indicating this function will not return
    // This function is called on panic. Here we simply print the panic info and halt.
    println!("{}", info);

    loop {
        // Infinite loop to halt the program
    }
}