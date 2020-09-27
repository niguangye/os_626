use x86_64::{structures::paging::PageTable, VirtAddr, };

pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)->&'static mut PageTable {
    use x86_64::registers::control::Cr3;

    // 从Cr3寄存器读取4级页表的物理帧
    let (level_4_table_frame, _) = Cr3::read();

    // 读取4级页表物理帧的开始地址
    let phys = level_4_table_frame.start_address();

    // 通过与偏移相加，获取页表物理帧对应的虚拟地址
    // 内核选用了偏移映射整个物理内存的访问页表的方式
    let virt = physical_memory_offset + phys.as_u64();

    // 将虚拟地址转化为裸指针
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    // 放回4级页表的可变引用
    // unsafe
    &mut *page_table_ptr
}
