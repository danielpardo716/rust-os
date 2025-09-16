use crate::{print, println};
use crate::{gdt, idle_loop};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use pic8259::ChainedPics;
use lazy_static::lazy_static;
use spin;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// Chained Programmable Interrupt Controllers. Example configuration:
///                      ____________                          ____________
/// Real Time Clock --> |            |   Timer -------------> |            |
/// ACPI -------------> |            |   Keyboard-----------> |            |      _____
/// Available --------> | Secondary  |----------------------> | Primary    |     |     |
/// Available --------> | Interrupt  |   Serial Port 2 -----> | Interrupt  |---> | CPU |
/// Mouse ------------> | Controller |   Serial Port 1 -----> | Controller |     |_____|
/// Co-Processor -----> |            |   Parallel Port 2/3 -> |            |
/// Primary ATA ------> |            |   Floppy disk -------> |            |
/// Secondary ATA ----> |____________|   Parallel Port 1----> |____________|
pub static PICS: spin::Mutex<ChainedPics> = 
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });      // Unsafe because incorrect offsets can cause undefined behavior

/// Enum representing indexes for interrupt variants
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard
}

/// InterruptIndex helper functions
impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

lazy_static! {
    /// Static IDT instance - load expects an IDT with 'static lifetime
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
    
        // Set exception handlers
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX)
        };
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);

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

/// Double fault exception handler
/// A double fault occurs when an exception occurs that doesn't have a handler
/// Handler is diverging - x86_64 architecture does not permit a return from double fault
extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

/// Page fault exception handler
/// Occurs when a page fault happens (e.g. accessing a page that is not mapped to physical memory)
extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode)
{
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed address: {:?}", Cr2::read());   // CR2 register contains the accessed virtual address that caused the fault
    println!("Error code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    idle_loop();
}

/// Timer interrupt handler function
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame)
{
    print!(".");

    // Figure out whether primary/secondary PIC sent the interrupt and send an EOI signal to the proper controller
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

/// Keyboard interrupt handler function
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame)
{
    use x86_64::instructions::port::Port;
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
            Keyboard::new(ScancodeSet1::new(), layouts::Us104Key, HandleControl::Ignore)
        );
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);     // PS/2 controller - I/O port 0x60
    let scancode: u8 = unsafe { port.read() };

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {                  // Translate scancode to Option<KeyEvent>
        if let Some(key) = keyboard.process_keyevent(key_event) {               // Translate KeyEvent to character (if possible)
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    // Figure out whether primary/secondary PIC sent the interrupt and send an EOI signal to the proper controller
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

#[test_case]
fn test_breakpoint_exception() {
    // Invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}