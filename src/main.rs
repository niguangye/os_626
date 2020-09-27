#![no_std] // 禁用标准库链接
#![no_main] // 告诉Rust编译器我们不使用预定义的入口点
#![feature(custom_test_frameworks)]
#![test_runner(os_626::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use os_626::println;
use bootloader::{ BootInfo, entry_point };
use os_626::memory::active_level_4_table;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {

    use os_626::memory::active_level_4_table;
    use x86_64::VirtAddr;

    println!("Hello World{}", "!");

    os_626::init();

    use x86_64::registers::control::Cr3;

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let l4_table = unsafe { active_level_4_table((phys_mem_offset)  )};

    for (i, entry) in l4_table.iter().enumerate(){
        use x86_64::structures::paging::PageTable;

        if !entry.is_unused(){
            println!("L4 Entry {}: {:?}", i, entry);
            let phys = entry.frame().unwrap().start_address();
            let virt = phys.as_u64() + boot_info.physical_memory_offset;
            let ptr = VirtAddr::new(virt).as_mut_ptr();
            let l3_table: &PageTable = unsafe { &*ptr } ;
            for (i, entry) in l3_table.iter().enumerate(){
                if !entry.is_unused(){
                    println!("L3 Entry {}: {:?}", i, entry);
                }
            }
        }

    }

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








