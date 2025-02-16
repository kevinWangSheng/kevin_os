#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kevin_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
mod serial;
use core::panic::PanicInfo;

use vga_buffer::{BUFFER_HEIGHT, WRITER};
use x86_64::structures::idt::Entry;
mod vga_buffer;

#[panic_handler]
#[cfg(not(test))] 
fn panic(_info: &PanicInfo) -> ! {
    println!("{}",_info);
    loop {}
}
#[allow(dead_code)]
static HELLO: &[u8] = b"Hello World!";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // let vbg_buffer = 0xb8000 as *mut u8;

    // for (i, &byte) in HELLO.iter().enumerate() {
    //     unsafe {
    //         *vbg_buffer.offset(i as isize * 2) = byte;
    //         *vbg_buffer.offset(i as isize * 2 + 1) = 0xf;
    //     }
    // }
    // vga_buffer::write_something();
    println!("hello world!!");
    kevin_os::init();
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    loop {}
}



#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use kevin_os::test_panic_handler;

    test_panic_handler(info)
}

#[test_case]
pub fn test_main(){
    serial_print!("main test");
}
