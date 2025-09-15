#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use rust_os::{exit_qemu, serial_print, serial_println, QemuExitCode};
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    /// Similar to lib IDT, but with custom double fault handler to exit QEMU
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(rust_os::gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}

pub fn init_test_idt() {
    TEST_IDT.load();
}

extern "x86-interrupt" fn test_double_fault_handler(_stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);

    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");

    rust_os::gdt::init();
    init_test_idt();

    // Trigger a stack overflow
    stack_overflow();

    panic!("Execution continued after stack overflow");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info)
}

/// Infinite recursion function to intentionally overflow the stack
#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();                   // For each recursion, the return address is pushed
    volatile::Volatile::new(0).read();  // Dummy volatile read - prevent tail recursion optimizations
}