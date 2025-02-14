#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kevin_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use kevin_os::{exit_qemu, serial_print, serial_println, QemuExitCode};
use core::panic::PanicInfo;


#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_println!("test some thing");
    exit_qemu(QemuExitCode::Succeed);
    // test_main();
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[test_case]
fn should_fail() {
    serial_print!("should_panic::should_fail...\t");
    assert_eq!(1, 1);
}


