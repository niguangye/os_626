#![no_std] // 禁用标准库链接
#![no_main] // 告诉Rust编译器我们不使用预定义的入口点
#![feature(custom_test_frameworks)]
#![test_runner(os_626::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use os_626::println;
use bootloader::{ BootInfo, entry_point };

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {

    println!("Hello World{}", "!");

    os_626::init();

    use x86_64::registers::control::Cr3;

    let (level_4_page_table, _) = Cr3::read();
    println!("Level 4 page table at: {:?}", level_4_page_table.start_address());

    #[cfg(test)]
    test_main();
    println!("It did not crash!");
    os_626::hlt_loop();
}

/// 这个函数将在panic时被调用
/// 类型为PanicInfo的参数包含了panic发生的文件名、代码行数和可选的错误信息
/// 这个函数从不返回，所以他被标记为发散函数（diverging function）。发散函数的返回类型称作Never类型（"never" type），记为!
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    os_626::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os_626::test_panic_handler(info)
}








