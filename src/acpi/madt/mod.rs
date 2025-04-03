use core::{cell::SyncUnsafeCell, mem};
use super::{find_sdt, sdt::Sdt};

/// The Multiple APIC Descriptor Table
#[derive(Clone, Copy, Debug)]
pub struct Madt {
    sdt: &'static Sdt,
    pub local_address: u32,
    pub flags: u32,
}

#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64.rs"]
mod arch;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[path = "arch/x86.rs"]
mod arch;

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86", target_arch = "x86_64")))]
#[path = "arch/other.rs"]
mod arch;

static MADT: SyncUnsafeCell<Option<Madt>> = SyncUnsafeCell::new(None);

pub fn madt() -> Option<&'static Madt> {
    // SAFETY: The `MADT` variable is initialized only once before use.
    unsafe { &*MADT.get() }.as_ref()
}

pub const FLAG_PCAT: u32 = 1;

impl Madt {
    pub fn init() {
        if let Some(madt_sdt) = find_sdt("APIC").first() {
            if let Some(madt) = Madt::new(madt_sdt) {
                // SAFETY: Ensuring single initialization before APs start.
                unsafe { MADT.get().write(Some(madt)) };
                println!("  APIC: {:>08X}: {}", madt.local_address, madt.flags);
                arch::init(madt);
            } else {
                println!("Invalid MADT structure.");
            }
        } else {
            println!("Unable to find MADT");
        }
    }

    pub fn new(sdt: &'static Sdt) -> Option<Madt> {
        if sdt.signature == *b"APIC" && sdt.data_len() >= 8 {
            let data_ptr = sdt.data_address() as *const u32;
            let (local_address, flags) = unsafe { (data_ptr.read_unaligned(), data_ptr.add(1).read_unaligned()) };
            Some(Madt { sdt, local_address, flags })
        } else {
            None
        }
    }

    pub fn iter(&self) -> MadtIter {
        MadtIter { sdt: self.sdt, i: 8 }
    }
}

/// MADT Iteration Structure
pub struct MadtIter {
    sdt: &'static Sdt,
    i: usize,
}

impl Iterator for MadtIter {
    type Item = MadtEntry;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.i + 1 >= self.sdt.data_len() {
            return None;
        }

        let base_ptr = self.sdt.data_address() as *const u8;
        let entry_type = unsafe { base_ptr.add(self.i).read() };
        let entry_len = unsafe { base_ptr.add(self.i + 1).read() } as usize;

        if self.i + entry_len > self.sdt.data_len() {
            return None;
        }

        let item = match entry_type {
            0x0 if entry_len == mem::size_of::<MadtLocalApic>() + 2 =>
                MadtEntry::LocalApic(unsafe { &*(base_ptr.add(self.i + 2) as *const MadtLocalApic) }),
            0x1 if entry_len == mem::size_of::<MadtIoApic>() + 2 =>
                MadtEntry::IoApic(unsafe { &*(base_ptr.add(self.i + 2) as *const MadtIoApic) }),
            0x2 if entry_len == mem::size_of::<MadtIntSrcOverride>() + 2 =>
                MadtEntry::IntSrcOverride(unsafe { &*(base_ptr.add(self.i + 2) as *const MadtIntSrcOverride) }),
            0xB if entry_len >= mem::size_of::<MadtGicc>() + 2 =>
                MadtEntry::Gicc(unsafe { &*(base_ptr.add(self.i + 2) as *const MadtGicc) }),
            0xC if entry_len >= mem::size_of::<MadtGicd>() + 2 =>
                MadtEntry::Gicd(unsafe { &*(base_ptr.add(self.i + 2) as *const MadtGicd) }),
            _ => MadtEntry::Unknown(entry_type),
        };

        self.i += entry_len;
        Some(item)
    }
}

/// MADT Entry Variants
#[derive(Debug)]
pub enum MadtEntry {
    LocalApic(&'static MadtLocalApic),
    IoApic(&'static MadtIoApic),
    IntSrcOverride(&'static MadtIntSrcOverride),
    Gicc(&'static MadtGicc),
    Gicd(&'static MadtGicd),
    Unknown(u8),
}

// Data structures for MADT Entries
#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct MadtLocalApic {
    pub processor: u8,
    pub id: u8,
    pub flags: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct MadtIoApic {
    pub id: u8,
    _reserved: u8,
    pub address: u32,
    pub gsi_base: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct MadtIntSrcOverride {
    pub bus_source: u8,
    pub irq_source: u8,
    pub gsi_base: u32,
    pub flags: u16,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct MadtGicc {
    _reserved: u16,
    pub cpu_interface_number: u32,
    pub acpi_processor_uid: u32,
    pub flags: u32,
    pub parking_protocol_version: u32,
    pub performance_interrupt_gsiv: u32,
    pub parked_address: u64,
    pub physical_base_address: u64,
    pub gicv: u64,
    pub gich: u64,
    pub vgic_maintenance_interrupt: u32,
    pub gicr_base_address: u64,
    pub mpidr: u64,
    pub processor_power_efficiency_class: u8,
    _reserved2: u8,
    pub spe_overflow_interrupt: u16,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct MadtGicd {
    _reserved: u16,
    pub gic_id: u32,
    pub physical_base_address: u64,
    pub system_vector_base: u32,
    pub gic_version: u8,
    _reserved2: [u8; 3],
}
