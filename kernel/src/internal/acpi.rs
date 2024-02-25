use alloc::boxed::Box;
use core::ptr::NonNull;
use acpi::{AcpiError, AcpiHandler, HpetInfo, InterruptModel, PciConfigRegions, PhysicalMapping, PlatformInfo, PowerProfile};
use acpi::fadt::Fadt;
use acpi::madt::Madt;
use acpi::platform::{PmTimer, ProcessorInfo};
use aml::{AmlContext, AmlName, AmlValue, DebugVerbosity};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::instructions::port::Port;
use crate::internal::aml::AmlHandler;

static mut PM1A_CNT_BLK: u32 = 0;
static mut SLP_TYPA: u16 = 0;
static SLP_LEN: u16 = 1 << 13;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformType {
    Unspecified,
    Desktop,
    Mobile,
    Workstation,
    EnterpriseServer,
    SohoServer,
    AppliancePc,
    PerformanceServer,
    Tablet
} impl From<PowerProfile> for PlatformType {
    fn from(profile: PowerProfile) -> Self {
        match profile {
            PowerProfile::Unspecified => PlatformType::Unspecified,
            PowerProfile::Desktop => PlatformType::Desktop,
            PowerProfile::Mobile => PlatformType::Mobile,
            PowerProfile::Workstation => PlatformType::Workstation,
            PowerProfile::EnterpriseServer => PlatformType::EnterpriseServer,
            PowerProfile::SohoServer => PlatformType::SohoServer,
            PowerProfile::AppliancePc => PlatformType::AppliancePc,
            PowerProfile::PerformanceServer => PlatformType::PerformanceServer,
            PowerProfile::Tablet => PlatformType::Tablet,
            PowerProfile::Reserved(_) => PlatformType::Unspecified
        }
    }
}

pub struct PlatformInfoWrapper<'a>(PlatformInfo<'a, alloc::alloc::Global>);
#[allow(dead_code)] impl<'a> PlatformInfoWrapper<'a> {
    pub fn new(info: PlatformInfo<'a, alloc::alloc::Global>) -> Self {
        Self(info)
    }
    pub fn platform_type(&self) -> PlatformType {
        PlatformType::from(self.0.power_profile)
    }

    pub fn interrupt_model(&self) -> &InterruptModel<alloc::alloc::Global> {
        &self.0.interrupt_model
    }

    pub fn processor_info(&self) -> Option<&ProcessorInfo<alloc::alloc::Global>> {
        self.0.processor_info.as_ref()
    }

    pub fn pm_timer(&self) -> Option<&PmTimer> {
        self.0.pm_timer.as_ref()
    }
}

pub struct Acpi {
    physical_memory_offset: VirtAddr,
    internal_tables: acpi::AcpiTables<MainAcpiHandler>,
    aml_handler: AmlHandler
} #[allow(dead_code)] impl Acpi {
    pub fn new(
        physical_memory_offset: VirtAddr,
        internal_tables: acpi::AcpiTables<MainAcpiHandler>
    ) -> Self { Self {
        physical_memory_offset,
        internal_tables,
        aml_handler: AmlHandler::new()
    } }

    pub fn platform_info(&self) -> Result<PlatformInfoWrapper, AcpiError> {
        match self.internal_tables.platform_info() {
            Ok(info) => Ok(PlatformInfoWrapper::new(info)),
            Err(err) => Err(err)
        }
    }

    pub fn hpet_info(&self) -> Result<HpetInfo, AcpiError> {
        HpetInfo::new(&self.internal_tables)
    }

    pub fn pci_config_regions(&self) -> Result<PciConfigRegions<alloc::alloc::Global>, AcpiError> {
        PciConfigRegions::new(&self.internal_tables)
    }

    pub fn fadt(&self) -> Result<&Fadt, AcpiError> {
        match self.internal_tables.find_table::<Fadt>() {
            Ok(mapping) => {
                Ok(unsafe { mapping.virtual_start().as_ref() })
            }, Err(err) => Err(err)
        }
    }

    pub fn madt(&self) -> Result<&Madt, AcpiError> {
        match self.internal_tables.find_table::<Madt>() {
            Ok(mapping) => {
                Ok(unsafe { mapping.virtual_start().as_ref() })
            }, Err(err) => Err(err)
        }
    }

    pub fn dsdt(&self) -> Result<&[u8], AcpiError> {
        let dsdt = match self.internal_tables.dsdt() {
            Ok(dsdt) => dsdt,
            Err(err) => return Err(err)
        };

        let phys_addr = PhysAddr::new(dsdt.address as u64);
        let virt_addr = crate::internal::memory::phys_to_virt(self.physical_memory_offset, phys_addr);
        let ptr: *const u8 = virt_addr.as_ptr();

        Ok(unsafe {
            core::slice::from_raw_parts(ptr, dsdt.length as usize)
        })
    }

    pub fn shutdown(&self) -> Result<(), AcpiError> {
        let dsdt_table = match self.dsdt() {
            Ok(dsdt) => dsdt,
            Err(err) => return Err(err)
        };
        let handler = Box::new(self.aml_handler.clone());
        let mut aml = AmlContext::new(handler, DebugVerbosity::None);
        if aml.parse_table(dsdt_table).is_ok() {
            let name = AmlName::from_str("\\_S5").unwrap();
            let res = aml.namespace.get_by_path(&name);
            if let Ok(AmlValue::Package(s5)) = res {
                if let AmlValue::Integer(value) = s5[0] {
                    unsafe {
                        SLP_TYPA = value as u16;
                    }
                }
            }
        } else {
            log::warn!("Failed to parse DSDT table for ACPI shutdown.");
            unsafe { SLP_TYPA = ( 5 & 7 ) << 10 }
        }

        unsafe {
            let mut port: Port<u16> = Port::new(PM1A_CNT_BLK as u16);
            port.write(SLP_TYPA | SLP_LEN);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

pub fn load(rsdp_addr: Option<u64>, physical_memory_offset: VirtAddr) -> Acpi {
    let handler = MainAcpiHandler::new(physical_memory_offset);

    let acpi_result = match rsdp_addr {
        Some(addr) => unsafe { acpi::AcpiTables::from_rsdp(handler, addr as usize) },
        None => unsafe { acpi::AcpiTables::search_for_rsdp_bios(handler) }
    };

    let acpi_tables = match acpi_result {
        Ok(acpi_tables) => acpi_tables,
        Err(err) => panic!("Failed to load ACPI tables: {:?}", err)
    };

    Acpi::new(physical_memory_offset, acpi_tables)
}