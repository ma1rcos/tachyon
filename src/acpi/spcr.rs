use core::mem;

use super::{find_sdt, sdt::Sdt, GenericAddressStructure};
use crate::{
    device::{
        serial::{SerialKind, COM1},
        uart_pl011,
    },
    memory::{map_device_memory, PhysicalAddress, PAGE_SIZE},
};

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct Spcr {
    pub header: Sdt,
    pub interface_type: u8,
    _reserved: [u8; 3],
    pub base_address: GenericAddressStructure,
    pub interrupt_type: u8,
    pub irq: u8,
    pub gsiv: u32,
    pub configured_baud_rate: u8,
    pub parity: u8,
    pub stop_bits: u8,
    pub flow_control: u8,
    pub terminal_type: u8,
    pub language: u8,
    pub pci_device_id: u16,
    pub pci_vendor_id: u16,
    pub pci_bus: u8,
    pub pci_device: u8,
    pub pci_function: u8,
    pub pci_flags: u32,
    pub pci_segment: u8,
}

impl Spcr {
    /// Initializes the SPCR table, mapping the serial device if supported.
    pub fn init() {
        let spcr_sdt = find_sdt("SPCR");

        let Some(spcr) = spcr_sdt
            .first()
            .and_then(|sdt| Spcr::new(sdt))
        else {
            log::warn!("Failed to locate or parse SPCR");
            return;
        };

        if spcr.base_address.address == 0 {
            // Serial is disabled
            return;
        }

        match (spcr.header.revision, spcr.interface_type) {
            (2.., 3) => Self::init_pl011(spcr),
            (1, unsupported) | (_, unsupported) => {
                log::warn!(
                    "SPCR revision {} unsupported interface type {}",
                    spcr.header.revision,
                    unsupported
                );
            }
        }
    }

    /// Maps and initializes the PL011 UART if address properties match.
    fn init_pl011(spcr: &Spcr) {
        let base = &spcr.base_address;

        if base.address_space == 0 && base.bit_width == 32 && base.bit_offset == 0 && base.access_size == 3 {
            let virt = unsafe { map_device_memory(PhysicalAddress::new(base.address as usize), PAGE_SIZE) };
            let serial_port = uart_pl011::SerialPort::new(virt.data(), false);
            *COM1.lock() = Some(SerialKind::Pl011(serial_port));
        } else {
            log::warn!("SPCR unsupported address for PL011 {:#x?}", base);
        }
    }

    /// Parses an SPCR table from an SDT, ensuring safe length checks.
    #[inline(always)]
    pub fn new(sdt: &'static Sdt) -> Option<&'static Spcr> {
        (sdt.signature == *b"SPCR" && (sdt.length as usize).checked_sub(mem::size_of::<Spcr>()).is_some())
            .then(|| unsafe { &*(sdt as *const Sdt as *const Spcr) })
    }
}