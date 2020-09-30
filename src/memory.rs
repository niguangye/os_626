use x86_64::{
    structures::paging::{
        PageTable, OffsetPageTable, Page, PhysFrame, Mapper, Size4KiB, FrameAllocator,
    },
    VirtAddr,
    PhysAddr
};

pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

// 返回4级页表的可变引用
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)->&'static mut PageTable {
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

    // 返回4级页表的可变引用
    // unsafe
    &mut *page_table_ptr
}


pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr)
    -> Option<PhysAddr> {
    translate_addr_inner(addr,physical_memory_offset)
}

fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr)
                        -> Option<PhysAddr>
{
    use x86_64::structures::paging::page_table::FrameError;
    use x86_64::registers::control::Cr3;

    // read the active level 4 frame from the CR3 register
    let (level_4_table_frame, _) = Cr3::read();

    let table_indexes = [
        addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
    ];
    let mut frame = level_4_table_frame;

    // traverse the multi-level page table
    for &index in &table_indexes {
        // convert the frame into a page table reference
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe {&*table_ptr};

        // read the page table entry and update `frame`
        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
        };
    }

    // calculate the physical address by adding the page offset
    Some(frame.start_address() + u64::from(addr.page_offset()))
}

// 创建从虚拟内存页到物理内存帧`0xb8000`的映射
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}
