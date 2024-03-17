use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use crate::{println, print, gdt, hlt_loop};
use pic8259::ChainedPics;
use spin;

pub const PIC_OFFSET_1: u8 = 32;
pub const PIC_OFFSET_2: u8 = PIC_OFFSET_1 + 8;

pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new( unsafe {
    ChainedPics::new(PIC_OFFSET_1, PIC_OFFSET_2) } );

lazy_static! {
static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
        idt.double_fault.set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);

        idt
        };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT!\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64)
    -> !{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    print!(".");
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(Keyboard::new(
            layouts::Us104Key, ScancodeSet1, HandleControl::Ignore));
    }

    let mut port = Port::new(0x60);
    let mut keyboard = KEYBOARD.lock();

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(char) => print!("{}", char),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame,
                                             error_code: PageFaultErrorCode) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("ACCESSING ADDRESS: {:?}", Cr2::read());
    println!("ERROR CODE: {:?}", error_code);
    println!("{:#?}", stack_frame);

    hlt_loop();
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_OFFSET_1,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        self as usize
    }
}

#[test_case]
fn test_breakpoint_exception() {
    // Invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}