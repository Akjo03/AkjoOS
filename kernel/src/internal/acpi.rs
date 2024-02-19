use core::ptr::NonNull;
use acpi::{AcpiHandler, AcpiTables, PhysicalMapping};
use x86_64::VirtAddr;

#[derive(Debug, Clone)]
pub struct MainAcpiHandler {
    physical_memory_offset: VirtAddr
} impl MainAcpiHandler {
    pub fn new(physical_memory_offset: VirtAddr) -> Self { Self {
        physical_memory_offset
    } }
} impl AcpiHandler for MainAcpiHandler {
    unsafe fn map_physical_region<T>(&self, physical_address: usize, size: usize) -> PhysicalMapping<Self, T> {
        let start_virt = self.physical_memory_offset.as_u64() + physical_address as u64;

        PhysicalMapping::new(
            physical_address,
            NonNull::new(VirtAddr::new(start_virt).as_mut_ptr()).unwrap(),
            size,
            size,
            self.clone()
        )
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}

pub fn load(rsdp_addr: Option<u64>, physical_memory_offset: VirtAddr) -> AcpiTables<MainAcpiHandler> {
    let handler = MainAcpiHandler::new(physical_memory_offset);

    let acpi_result = match rsdp_addr {
        Some(addr) => unsafe { AcpiTables::from_rsdp(handler, addr as usize) },
        None => unsafe { AcpiTables::search_for_rsdp_bios(handler) }
    };

    match acpi_result {
        Ok(acpi_tables) => acpi_tables,
        Err(err) => panic!("ACPI error: {:?}", err)
    }
}