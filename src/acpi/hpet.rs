use core::{mem, ptr};
use core::ptr::{read_volatile, write_volatile};

use crate::memory::{map_device_memory, PhysicalAddress, PAGE_SIZE};

use super::{find_sdt, sdt::Sdt, GenericAddressStructure, ACPI_TABLE};

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct Hpet {
    pub header: Sdt,
    pub hw_rev_id: u8,
    pub comparator_descriptor: u8,
    pub pci_vendor_id: u16,
    pub base_address: GenericAddressStructure,
    pub hpet_number: u8,
    pub min_periodic_clk_tick: u16,
    pub oem_attribute: u8,
}

impl Hpet {
    #[inline(always)]
    pub fn init() {
        let hpet = Hpet::new(find_sdt("HPET").get(0)?)?;

        log::info!("  HPET: {:X}", hpet.hpet_number);
        *ACPI_TABLE.hpet.write() = Some(hpet);
    }

    #[inline(always)]
    pub fn new(sdt: &'static Sdt) -> Option<&'static Hpet> {
        (sdt.signature == *b"HPET" && sdt.length as usize >= mem::size_of::<Hpet>())
            .then(|| unsafe { &*ptr::cast::<_, Hpet>(sdt) })
            .filter(|h| h.base_address.address_space == 0)
            .map(|h| {
                unsafe { h.map().ok()? };
                h
            })
    }
}

#[cfg(target_arch = "x86")]
impl Hpet {
    #[inline(always)]
    pub unsafe fn map(&self) -> Result<(), &'static str> {
        use crate::{
            memory::{Frame, KernelMapper},
            paging::{entry::EntryFlags, Page, VirtualAddress},
        };
        use rmm::PageFlags;

        let frame = Frame::containing(PhysicalAddress::new(self.base_address.address as usize));
        let page = Page::containing_address(VirtualAddress::new(crate::HPET_OFFSET));

        KernelMapper::lock()
            .get_mut()
            .ok_or("KernelMapper already locked")?
            .map_phys(
                page.start_address(),
                frame.base(),
                PageFlags::new()
                    .write(true)
                    .custom_flag(EntryFlags::NO_CACHE.bits(), true),
            )?
            .flush();

        Ok(())
    }

    #[inline(always)]
    pub unsafe fn read_u64(&self, offset: usize) -> u64 {
        read_volatile((crate::HPET_OFFSET + offset) as *const u64)
    }

    #[inline(always)]
    pub unsafe fn write_u64(&mut self, offset: usize, value: u64) {
        write_volatile((crate::HPET_OFFSET + offset) as *mut u64, value);
    }
}

#[cfg(not(target_arch = "x86"))]
impl Hpet {
    #[inline(always)]
    pub unsafe fn map(&self) -> Result<(), ()> {
        map_device_memory(
            PhysicalAddress::new(self.base_address.address as usize),
            PAGE_SIZE,
        );
        Ok(())
    }

    #[inline(always)]
    pub unsafe fn read_u64(&self, offset: usize) -> u64 {
        read_volatile(
            (self.base_address.address as usize + offset + crate::PHYS_OFFSET) as *const u64,
        )
    }

    #[inline(always)]
    pub unsafe fn write_u64(&mut self, offset: usize, value: u64) {
        write_volatile(
            (self.base_address.address as usize + offset + crate::PHYS_OFFSET) as *mut u64,
        );
    }
}