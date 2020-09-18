#![no_std] // 禁用标准库链接
#![no_main] // 告诉Rust编译器我们不使用预定义的入口点
#![feature(custom_test_frameworks)]
#![test_runner(os_626::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use os_626::println;

#[no_mangle] // 禁用名称重整，确保Rust编译器输出一个名为_start的函数；否则编译器可能最终生成名为_ZN3blog_os4_start7hb173fedf945531caE的函数，无法让链接器正确辨别。
// extern "C" 告诉编译器这个函数应当使用C语言的调用约定
pub extern "C" fn _start() -> ! {

    println!("Hello World{}", "!");

    os_626::init();

/*    fn stack_overflow(){
        stack_overflow();
    }

    stack_overflow();*/

    #[cfg(test)]
    test_main();
    println!("It did not crash!");
    loop {}
}

/// 这个函数将在panic时被调用
/// 类型为PanicInfo的参数包含了panic发生的文件名、代码行数和可选的错误信息
/// 这个函数从不返回，所以他被标记为发散函数（diverging function）。发散函数的返回类型称作Never类型（"never" type），记为!
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os_626::test_panic_handler(info)
}








