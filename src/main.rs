#![no_std] // 禁用标准库链接
#![no_main] // 告诉Rust编译器我们不使用预定义的入口点

use core::panic::PanicInfo;

static HELLO: &[u8] = b"Hello World!";

#[no_mangle] // 禁用名称重整，确保Rust编译器输出一个名为_start的函数；否则编译器可能最终生成名为_ZN3blog_os4_start7hb173fedf945531caE的函数，无法让链接器正确辨别。
// extern "C" 告诉编译器这个函数应当使用C语言的调用约定
pub extern "C" fn _start() -> ! {

    ///我们预先定义了一个字节串（byte string）类型的静态变量（static variable），名为HELLO。
    ///我们首先将整数0xb8000转换（cast）为一个裸指针（raw pointer）。
    ///这之后，我们迭代HELLO的每个字节，使用enumerate获得一个额外的序号变量i。
    ///在for语句的循环体中，我们使用offset偏移裸指针，解引用它，来将字符串的每个字节和对应的颜色字节——0xb代表淡青色——写入内存位置。
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }

    loop {}

    loop {}
}

/// 这个函数将在panic时被调用
/// 类型为PanicInfo的参数包含了panic发生的文件名、代码行数和可选的错误信息
/// 这个函数从不返回，所以他被标记为发散函数（diverging function）。发散函数的返回类型称作Never类型（"never" type），记为!
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}


