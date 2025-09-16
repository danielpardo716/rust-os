#![no_std]                                      // Don't link the Rust standard library
#![no_main]                                     // Don't use the Rust main function as the entry point
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
// #![warn(missing_docs)]

use core::panic::PanicInfo;
use rust_os::println;

#[unsafe(no_mangle)]                                 // Ensure compiler keeps the name of this function as "_start" as this is the linker-defined entry point
pub extern "C" fn _start() -> ! {                    // Extern "C" - use the C calling convention for starting point instead of Rust calling convention  
    println!("Hello World{}", "!");

    rust_os::init();

    use x86_64::registers::control::Cr3;
    let (level_4_page_table, _) = Cr3::read();
    println!("Level 4 page table at: {:?}", level_4_page_table.start_address());
    
    // If compiled in test mode, run the tests.
    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    rust_os::idle_loop();
}

/// Default Panic handler for the application.
// Since we are using `no_std`, we need to define a panic handler.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {           // ! is the "never" type, indicating this function will not return
    // This function is called on panic. Here we simply print the panic info and halt.
    println!("{}", info);
    rust_os::idle_loop();
}

/// Panic handler for test runs - use serial port instead of VGA buffer
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {                          // ! is the "never" type, indicating this function will not return
    // Call the library-provided test panic handler
    rust_os::test_panic_handler(info)
}