use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::println;
use lazy_static::lazy_static;

lazy_static! {
    /// Static IDT instance - load expects an IDT with 'static lifetime
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
    
        // Set exception handlers
        idt.breakpoint.set_handler_fn(breakpoint_handler);

        idt
    };
}

pub fn idt_init() {
    IDT.load();
}

/// Breakpoint exception handler
/// extern "x86-interrupt" specifies the calling convention for interrupt handlers
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame)
{
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

#[test_case]
fn test_breakpoint_exception() {
    // Invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}