//! # ACPI
//! Code to parse the ACPI tables

use alloc::{boxed::Box, string::String, vec::Vec};
use core::convert::TryFrom;

use hashbrown::HashMap;
use spin::{Once, RwLock};
use log::info;

use crate::{
    memory::KernelMapper,
    paging::{PageFlags, PhysicalAddress, RmmA, RmmArch},
};

use self::{hpet::Hpet, madt::Madt, rsdp::RSDP, rsdt::Rsdt, rxsdt::Rxsdt, sdt::Sdt, xsdt::Xsdt};

#[cfg(target_arch = "aarch64")]
mod gtdt;
pub mod hpet;
pub mod madt;
mod rsdp;
mod rsdt;
mod rxsdt;
pub mod sdt;
#[cfg(target_arch = "aarch64")]
mod spcr;
mod xsdt;

/// Safely maps a physical address range linearly into virtual memory.
unsafe fn map_linearly(addr: PhysicalAddress, len: usize, mapper: &mut crate::paging::PageMapper) {
    let base = PhysicalAddress::new(crate::paging::round_down_pages(addr.data()));
    let aligned_len = crate::paging::round_up_pages(len + addr.data().saturating_sub(base.data()));

    for page_idx in 0..aligned_len / crate::memory::PAGE_SIZE {
        if let Ok((_, flush)) = mapper.map_linearly(
            base.add(page_idx * crate::memory::PAGE_SIZE),
            PageFlags::new(),
        ) {
            flush.flush();
        } else {
            log::error!("Failed to linearly map SDT at {:#x}", addr.data());
        }
    }
}

/// Retrieves an `Sdt` from a physical address, ensuring safe memory mapping.
pub fn get_sdt(sdt_address: usize, mapper: &mut KernelMapper) -> Option<&'static Sdt> {
    let mapper = mapper.get_mut().expect("KernelMapper locked re-entrant in get_sdt");
    let physaddr = PhysicalAddress::new(sdt_address);

    unsafe {
        const SDT_SIZE: usize = core::mem::size_of::<Sdt>();
        map_linearly(physaddr, SDT_SIZE, mapper);

        let sdt = &*(RmmA::phys_to_virt(physaddr).data() as *const Sdt);
        let total_size = usize::try_from(sdt.length).ok()?;

        map_linearly(physaddr.add(SDT_SIZE), total_size.saturating_sub(SDT_SIZE), mapper);

        Some(sdt)
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default)]
pub struct GenericAddressStructure {
    pub address_space: u8,
    pub bit_width: u8,
    pub bit_offset: u8,
    pub access_size: u8,
    pub address: u64,
}

pub enum RxsdtEnum {
    Rsdt(Rsdt),
    Xsdt(Xsdt),
}

impl Rxsdt for RxsdtEnum {
    fn iter(&self) -> Box<dyn Iterator<Item = usize>> {
        match self {
            Self::Rsdt(rsdt) => rsdt.iter(),
            Self::Xsdt(xsdt) => xsdt.iter(),
        }
    }
}

pub static RXSDT_ENUM: Once<RxsdtEnum> = Once::new();

/// Parses the ACPI tables to gather CPU, interrupt, and timer information.
pub unsafe fn init(already_supplied_rsdp: Option<*const u8>) {
    SDT_POINTERS.write().replace(HashMap::new());

    let rsdp_opt = RSDP::get_rsdp(&mut KernelMapper::lock(), already_supplied_rsdp);
    let rsdp = match rsdp_opt {
        Some(r) => r,
        None => {
            println!("NO RSDP FOUND");
            return;
        }
    };

    info!("RSDP: {:?}", rsdp);
    let rxsdt = match get_sdt(rsdp.sdt_address(), &mut KernelMapper::lock()) {
        Some(s) => s,
        None => {
            log::error!("Failed to get RSDT/XSDT");
            return;
        }
    };

    let rx_enum = match (Rsdt::new(rxsdt), Xsdt::new(rxsdt)) {
        (Some(rsdt), _) => RXSDT_ENUM.call_once(|| RxsdtEnum::Rsdt(rsdt)),
        (_, Some(xsdt)) => RXSDT_ENUM.call_once(|| RxsdtEnum::Xsdt(xsdt)),
        _ => {
            println!("UNKNOWN RSDT OR XSDT SIGNATURE");
            return;
        }
    };

    for sdt_addr in rx_enum.iter() {
        if let Some(sdt) = get_sdt(sdt_addr, &mut KernelMapper::lock()) {
            let signature = get_sdt_signature(sdt);
            SDT_POINTERS.write().as_mut().unwrap().insert(signature, sdt);
        }
    }

    #[cfg(target_arch = "aarch64")]
    spcr::Spcr::init();
    Madt::init();
    Hpet::init();
    #[cfg(target_arch = "aarch64")]
    gtdt::Gtdt::init();
}

pub type SdtSignature = (String, [u8; 6], [u8; 8]);
pub static SDT_POINTERS: RwLock<Option<HashMap<SdtSignature, &'static Sdt>>> = RwLock::new(None);

/// Finds an SDT by name, returning a vector of matching tables.
pub fn find_sdt(name: &str) -> Vec<&'static Sdt> {
    SDT_POINTERS
        .read()
        .as_ref()
        .map_or_else(Vec::new, |ptrs| {
            ptrs.iter()
                .filter_map(|(signature, sdt)| (signature.0 == name).then_some(*sdt))
                .collect()
        })
}

/// Extracts the signature from an SDT.
pub fn get_sdt_signature(sdt: &'static Sdt) -> SdtSignature {
    let signature = String::from_utf8_lossy(&sdt.signature).into_owned();
    (signature, sdt.oem_id, sdt.oem_table_id)
}

pub struct Acpi {
    pub hpet: RwLock<Option<Hpet>>,
    pub next_ctx: RwLock<u64>,
}

pub static ACPI_TABLE: Acpi = Acpi {
    hpet: RwLock::new(None),
    next_ctx: RwLock::new(0),
};