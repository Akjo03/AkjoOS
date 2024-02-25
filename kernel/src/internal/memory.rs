use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use x86_64::structures::paging::{OffsetPageTable, PageTable, PhysFrame};
use x86_64::{PhysAddr, VirtAddr};

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = phys_to_virt(physical_memory_offset, phys);
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

pub fn get_usable_regions(memory_regions: &'static MemoryRegions, skip: usize) -> impl Iterator<Item = PhysFrame> {
    memory_regions.iter()
        .filter(|region| region.kind == MemoryRegionKind::Usable)
        .filter(|region| region.start % 4096 == 0 && region.end % 4096 == 0)
        .map(|region| region.start..region.end)
        .flat_map(|region_range| region_range.step_by(4096))
        .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
        .skip(skip)
}

pub fn phys_to_virt(physical_memory_offset: VirtAddr, physical_address: PhysAddr) -> VirtAddr {
    physical_memory_offset + physical_address.as_u64()
}

#[allow(dead_code)]
pub fn read_address<T>(address: usize) -> T where T: Copy {
    let virt_addr = VirtAddr::new(address as u64);

    unsafe { *virt_addr.as_ptr::<T>() }
}

#[allow(dead_code)]
pub fn write_address<T>(address: usize, value: T) where T: Copy {
    let virt_addr = VirtAddr::new(address as u64);

    unsafe { *virt_addr.as_mut_ptr::<T>() = value; }
}