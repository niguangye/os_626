#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os_626::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use os_626::{println, serial_print, serial_println};

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    unimplemented!();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os_626::test_panic_handler(info)
}

#[test_case]
fn test_println() {
    serial_print!("test_println... ");
    println!("test_println output");
    serial_println!("[ok]");
}
