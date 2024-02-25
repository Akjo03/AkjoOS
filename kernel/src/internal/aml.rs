use aml::Handler;

#[derive(Clone)]
pub struct AmlHandler;
impl AmlHandler {
    pub fn new() -> Self { Self }
} impl Handler for AmlHandler {
    fn read_u8(&self, address: usize) -> u8 {
        crate::internal::memory::read_address::<u8>(address)
    }

    fn read_u16(&self, address: usize) -> u16 {
        crate::internal::memory::read_address::<u16>(address)
    }

    fn read_u32(&self, address: usize) -> u32 {
        crate::internal::memory::read_address::<u32>(address)
    }

    fn read_u64(&self, address: usize) -> u64 {
        crate::internal::memory::read_address::<u64>(address)
    }

    fn write_u8(&mut self, address: usize, value: u8) {
        crate::internal::memory::write_address::<u8>(address, value);
    }

    fn write_u16(&mut self, address: usize, value: u16) {
        crate::internal::memory::write_address::<u16>(address, value);
    }

    fn write_u32(&mut self, address: usize, value: u32) {
        crate::internal::memory::write_address::<u32>(address, value);
    }

    fn write_u64(&mut self, address: usize, value: u64) {
        crate::internal::memory::write_address::<u64>(address, value);
    }

    fn read_io_u8(&self, _port: u16) -> u8 { unimplemented!() }

    fn read_io_u16(&self, _port: u16) -> u16 { unimplemented!() }

    fn read_io_u32(&self, _port: u16) -> u32 { unimplemented!() }

    fn write_io_u8(&self, _port: u16, _value: u8) { unimplemented!() }

    fn write_io_u16(&self, _port: u16, _value: u16) { unimplemented!() }

    fn write_io_u32(&self, _port: u16, _value: u32) { unimplemented!() }

    fn read_pci_u8(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16) -> u8 { unimplemented!() }

    fn read_pci_u16(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16) -> u16 { unimplemented!() }

    fn read_pci_u32(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16) -> u32 { unimplemented!() }

    fn write_pci_u8(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16, _value: u8) { unimplemented!() }

    fn write_pci_u16(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16, _value: u16) { unimplemented!() }

    fn write_pci_u32(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16, _value: u32) { unimplemented!() }
}