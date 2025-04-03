use core::ptr::read_unaligned;
use crate::{
    memory::{Frame, KernelMapper},
    paging::{Page, PageFlags, PhysicalAddress, VirtualAddress},
};

/// RSDP (Root System Description Pointer)
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct RSDP {
    signature: [u8; 8],
    _checksum: u8,
    _oemid: [u8; 6],
    revision: u8,
    rsdt_address: u32,
    _length: u32,
    xsdt_address: u64,
    _extended_checksum: u8,
    _reserved: [u8; 3],
}

impl RSDP {
    /// Obtains an already provided RSDP, validating its integrity
    #[inline(always)]
    fn get_already_supplied_rsdp(rsdp_ptr: *const u8) -> Option<&'static RSDP> {
        let rsdp = unsafe { &*(rsdp_ptr as *const RSDP) };
        rsdp.validate_checksum().then_some(rsdp)
    }

    /// Gets the RSDP, searching memory if necessary
    pub fn get_rsdp(
        mapper: &mut KernelMapper,
        already_supplied_rsdp: Option<*const u8>,
    ) -> Option<&'static RSDP> {
        already_supplied_rsdp
            .and_then(Self::get_already_supplied_rsdp)
            .or_else(|| Self::get_rsdp_by_searching(mapper))
    }

    /// RSDP search in known address range
    pub fn get_rsdp_by_searching(mapper: &mut KernelMapper) -> Option<&'static RSDP> {
        const START_ADDR: usize = 0xE_0000;
        const END_ADDR: usize = 0xF_FFFF;

        // Securely maps the ACPI region
        let frames = Frame::range_inclusive(
            Frame::containing(PhysicalAddress::new(START_ADDR)),
            Frame::containing(PhysicalAddress::new(END_ADDR)),
        );

        for frame in frames {
            let page = Page::containing_address(VirtualAddress::new(frame.base().data()));
            unsafe {
                mapper.get_mut()?.map_phys(page.start_address(), frame.base(), PageFlags::new()).ok()?;
            }
        }

        Self::search(START_ADDR, END_ADDR)
    }

    /// Search for RSDP within the mapped region
    fn search(start_addr: usize, end_addr: usize) -> Option<&'static RSDP> {
        (start_addr..=end_addr)
            .step_by(16)
            .map(|addr| unsafe { &*read_unaligned(addr as *const RSDP) })
            .find(|rsdp| rsdp.signature == *b"RSD PTR " && rsdp.validate_checksum())
    }

    /// Returns the address of the root table (XSDT or RSDT)
    #[inline(always)]
    pub fn sdt_address(&self) -> usize {
        if self.revision >= 2 {
            self.xsdt_address as usize
        } else {
            self.rsdt_address as usize
        }
    }

    /// Validates the RSDP checksum
    fn validate_checksum(&self) -> bool {
        let bytes = unsafe { core::slice::from_raw_parts(self as *const _ as *const u8, 20) };
        bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b)) == 0
    }

}