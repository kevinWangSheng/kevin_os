#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kevin_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use kevin_os::{println, serial_println};


#[panic_handler]
pub fn panic(info:&PanicInfo)->!{
    kevin_os::test_panic_handler(info);
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}


#[test_case]
pub fn test_println(){
    println!("wangwu");
    serial_println!("wangwu12312");
}

#[test_case]
pub fn test_println(){
    println!("wangwu");
    serial_println!("sadfasdfasd");
}

