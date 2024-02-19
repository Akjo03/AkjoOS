use alloc::vec::Vec;
use acpi::AcpiTables;
use acpi::madt::{Madt, MadtEntry, MadtEntryIter};
use x86_64::{PhysAddr, VirtAddr};
use crate::internal::acpi::MainAcpiHandler;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MadtEntryType {
    LocalApic(LocalApic),
    IoApic(IoApic)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalApic {
    pub entry_type: u8,
    pub entry_length: u8,
    pub processor_id: u8,
    pub apic_id: u8,
    pub flags: u32,
    pub address: u32
} impl LocalApic {
    pub fn new(
        entry_type: u8, entry_length: u8,
        processor_id: u8, apic_id: u8, flags: u32, address: u32
    ) -> Self { Self {
        entry_type, entry_length,
        processor_id, apic_id, flags, address
    } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IoApic {
    pub entry_type: u8,
    pub entry_length: u8,
    pub apic_id: u8,
    pub address: u32,
    pub global_system_interrupt_base: u32,
    physical_memory_offset: VirtAddr
} impl IoApic {
    pub fn new(
        entry_type: u8, entry_length: u8,
        apic_id: u8, address: u32, global_system_interrupt_base: u32,
        physical_memory_offset: VirtAddr
    ) -> Self { Self {
        entry_type, entry_length,
        apic_id, address, global_system_interrupt_base,
        physical_memory_offset
    } }

    pub fn address(&self) -> VirtAddr {
        VirtAddr::new(
            self.physical_memory_offset.as_u64()
                + self.address as u64
        )
    }
}

pub struct MadtTable {
    madt_entries: Vec<MadtEntryType>,
    physical_memory_offset: VirtAddr,
} impl MadtTable {
    pub fn new(
        local_apic_address: u32,
        madt_entries: MadtEntryIter,
        physical_memory_offset: VirtAddr
    ) -> Self {
        let mut madt_entry_vec = Vec::new();
        madt_entries.for_each(|madt_entry| { match madt_entry {
            MadtEntry::LocalApic(local_apic) => {
                madt_entry_vec.push(MadtEntryType::LocalApic(LocalApic::new(
                    local_apic.header.entry_type, local_apic.header.length,
                    local_apic.processor_id, local_apic.apic_id, local_apic.flags, local_apic_address
                )));
            }, MadtEntry::IoApic(io_apic) => {
                madt_entry_vec.push(MadtEntryType::IoApic(IoApic::new(
                    io_apic.header.entry_type, io_apic.header.length,
                    io_apic.io_apic_id, io_apic.io_apic_address, io_apic.global_system_interrupt_base,
                    physical_memory_offset
                )));
            }, _ => {}
        }});
        Self { madt_entries: madt_entry_vec, physical_memory_offset }
    }

    pub fn local_apic(&self) -> Option<&LocalApic> {
        self.madt_entries.iter().find_map(|madt_entry| {
            if let MadtEntryType::LocalApic(local_apic) = madt_entry {
                Some(local_apic)
            } else { None }
        })
    }

    pub fn phys_lapic_addr(&self) -> PhysAddr {
        PhysAddr::new(
            self.local_apic().expect("Failed to find local APIC!").address as u64
        )
    }

    pub fn virt_lapic_addr(&self) -> VirtAddr {
        VirtAddr::new(
            self.physical_memory_offset.as_u64()
                + self.local_apic().expect("Failed to find local APIC!").address as u64
        )
    }

    pub fn io_apics(&self) -> Vec<&IoApic> {
        self.madt_entries.iter().filter_map(|madt_entry| {
            if let MadtEntryType::IoApic(io_apic) = madt_entry {
                Some(io_apic)
            } else { None }
        }).collect()
    }
}

pub fn load(acpi_tables: &AcpiTables<MainAcpiHandler>, physical_memory_offset: VirtAddr) -> MadtTable {
    let madt_table = acpi_tables.find_table::<Madt>()
        .expect("Failed to find MADT table!");

    MadtTable::new(
        madt_table.local_apic_address,
        madt_table.entries(),
        physical_memory_offset
    )
}