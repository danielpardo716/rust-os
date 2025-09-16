#![no_std]
#![cfg_attr(test, no_main)]                     // Conditionally enable no_main for test runs
#![feature(custom_test_frameworks)]             // Enable custom test frameworks for our no_std environment
#![test_runner(crate::test_runner)]             // Define the test runner function to use for running tests
#![reexport_test_harness_main = "test_main"]    // Since we don't have a main function, rename the test harness entry point to "test_main"
#![feature(abi_x86_interrupt)]                  // Enable the x86-interrupt calling convention for interrupt handlers

use core::panic::PanicInfo;

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

extern crate alloc;

pub mod serial;
pub mod vga_buffer;
pub mod interrupts;
pub mod gdt;
pub mod memory;
pub mod allocator;

#[cfg(test)]
entry_point!(test_kernel_main);

/// Trait to have test_runner to automatically print testing statements
pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T 
where 
    T:Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {           // Test runner function that takes a slice of test functions
    serial_println!("Running {} tests...", tests.len());
    
    for test in tests {
        test.run();
    }
    
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    
    idle_loop();
}

/// Entry point for `cargo test`
#[cfg(test)]
pub fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();

    idle_loop();
}

/// Initialize all components of the OS
pub fn init() {
    gdt::init();
    interrupts::idt_init();
    unsafe {
        interrupts::PICS.lock().initialize()    // Unsafe - undefined behavior if PIC is misconfigured
    };
    x86_64::instructions::interrupts::enable();
}

/// Idle loop to wait until next interrupt. Causes CPU to enter sleep, consuming less energy.
pub fn idle_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/// Panic handler for test runs - use serial port instead of VGA buffer
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {                          // ! is the "never" type, indicating this function will not return
    test_panic_handler(info);
}

/// Exit codes for QEMU to indicate success or failure of the tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

/// Exit QEMU with the given exit code.
/// This function uses the `isa-debug-exit` device to signal QEMU to exit.
/// The exit code is sent to the I/O port `0xf4`, which is configured in the QEMU command line arguments (see `Cargo.toml`).
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;
    unsafe {
        let mut port = Port::new(0xf4);    // Create port at iobase (specified in Cargo.toml)
        port.write(exit_code as u32);                                               // iosize is 4 bytes
    }
}