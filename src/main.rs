#![no_std]                                      // Don't link the Rust standard library
#![no_main]                                     // Don't use the Rust main function as the entry point
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
// #![warn(missing_docs)]

use core::panic::PanicInfo;
use bootloader::{entry_point, BootInfo};
use rust_os::println;

entry_point!(kernel_main);               // Define the entry point function for the kernel

/// The entry point for the kernel, called by the bootloader.
/// entry_point macro ensures the function has the correct signature and creates the underlying extern "C" _start function.
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use rust_os::memory;
    use x86_64::{VirtAddr, structures::paging::Page};

    println!("Hello World{}", "!");
    rust_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe{ memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };
    
    // Map an unused page
    let page = Page::containing_address(VirtAddr::new(0));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // Write the string "New!" to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) }; // "New!"

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