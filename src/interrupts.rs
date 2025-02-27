use core::panic;

use lazy_static::lazy_static;

use pc_keyboard::{DecodedKey, HandleControl, KeyboardLayout, ScancodeSet, ScancodeSet1};
use spin::Mutex;
use x86_64::{
    instructions::port::Port,
    structures::{idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode}, paging::PageTable},
};

use crate::{exit_qemu, gdt, hlt_loop, print, println};

pub const PIC_1_OFFER: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFER + 8;

pub const PICS: Mutex<pic8259::ChainedPics> =
    Mutex::new(unsafe { pic8259::ChainedPics::new(PIC_1_OFFER, PIC_2_OFFSET) });
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::KeyBoard as usize].set_handler_fn(keyboard_interrupt_handle);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt
    };
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFER,
    KeyBoard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub fn init_idt() {
    IDT.load();
}

pub fn init_pic() {
    unsafe {
        PICS.lock().initialize();
    }
    x86_64::instructions::interrupts::enable();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(stack_frame: InterruptStackFrame) {
    // println!("timer ..");
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer as u8);
    }
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame,error_code:PageFaultErrorCode){
    use x86_64::registers::control::Cr2;
    println!("EXCEPTION:PAGE FAULT");
    println!("Accessed Address:{:?}",Cr2::read());
    println!("Error Code :{:?}",error_code);
    println!("{:#?}",stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn keyboard_interrupt_handle(stack_frame: InterruptStackFrame) {
    // the keyboard I/O port is the 0x60
    use pc_keyboard::layouts;
    use pc_keyboard::Keyboard;
    use x86_64::instructions::port::Port;
    // initialize the mutex keyboard and use lazy_static
    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(
                ScancodeSet1::new(),
                layouts::Us104Key,
                HandleControl::Ignore
            ));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key{
                DecodedKey::Unicode(character)=>print!("{}",character),
                DecodedKey::RawKey(key)=>print!("{:?}",key)
            }
        }
    }
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::KeyBoard as u8);
    }
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _code: u64) -> ! {
    panic!("EXCEPTION DOUBLE FAULT \n{:#?}", stack_frame);
    // exit_qemu(crate::QemuExitCode::Succeed);
    // loop {}
}

#[test_case]
fn test_breakpoint_exception() {
    // push to the instructions of the int3 to create a breakpoint exception
    x86_64::instructions::interrupts::int3();
}
