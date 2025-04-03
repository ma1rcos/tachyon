use core::mem;

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct Sdt {
    pub signature: [u8; 4],  // ACPI table signature
    pub length: u32,         // Total table length
    pub revision: u8,        // Revision number
    pub checksum: u8,        // Checksum for validation
    pub oem_id: [u8; 6],     // OEM identifier
    pub oem_table_id: [u8; 8], // OEM-specific table identifier
    pub oem_revision: u32,   // OEM revision number
    pub creator_id: u32,     // Creator ID
    pub creator_revision: u32, // Creator revision number
}

impl Sdt {
    /// Returns the starting address of the table's data section.
    #[inline(always)] // Encourages inlining for performance
    pub fn data_address(&self) -> usize {
        (self as *const Self as usize) + mem::size_of::<Sdt>()
    }

    /// Returns the length of the data section, ensuring safety.
    #[inline(always)]
    pub fn data_len(&self) -> usize {
        let total_size = self.length as usize;
        let header_size = mem::size_of::<Sdt>();

        total_size.checked_sub(header_size).unwrap_or(0)
    }
}