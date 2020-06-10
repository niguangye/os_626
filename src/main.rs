#![no_std] // 禁用标准库链接

use core::panic::PanicInfo;

fn main() {

}

/// 这个函数将在panic时被调用
/// 类型为PanicInfo的参数包含了panic发生的文件名、代码行数和可选的错误信息
/// 这个函数从不返回，所以他被标记为发散函数（diverging function）。发散函数的返回类型称作Never类型（"never" type），记为!
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}


