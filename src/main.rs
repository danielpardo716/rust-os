#![no_std]                                      // Don't link the Rust standard library
#![no_main]                                     // Don't use the Rust main function as the entry point
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
// #![warn(missing_docs)]

extern crate alloc;

use core::{panic::PanicInfo};
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};
use bootloader::{entry_point, BootInfo};
use rust_os::{
    println,
    task::{executor::Executor, keyboard, simple_executor::SimpleExecutor, Task}
};

entry_point!(kernel_main);               // Define the entry point function for the kernel

/// The entry point for the kernel, called by the bootloader.
/// entry_point macro ensures the function has the correct signature and creates the underlying extern "C" _start function.
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use rust_os::allocator;
    use rust_os::memory;
    use x86_64::{VirtAddr};

    println!("Hello World{}", "!");
    rust_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe{ memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::heap_init(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    // Allocate a number on the heap to test the allocator.
    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    // Create a dynamically sized vector.
    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference));

    // Test our multitasking executor
    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

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

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}