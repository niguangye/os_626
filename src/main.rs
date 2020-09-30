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

    use os_626::memory;
    use x86_64::{structures::paging::Page, VirtAddr};

    println!("Hello World{}", "!");

    os_626::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    let mut mapper = unsafe { memory::init(phys_mem_offset)};

    let mut frame_allocator = memory::EmptyFrameAllocator;

    let page = Page::containing_address(VirtAddr::new(0));

    memory::create_example_mapping(page,&mut mapper, &mut frame_allocator);

    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();

    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)}; // 0x_f021_f077_f065_f04e代表字符串 New！

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








