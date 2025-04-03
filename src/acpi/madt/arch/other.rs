use super::Madt;

// Initialize MADT (Multiple APIC Descriptor Table)
pub(super) fn init(madt: Madt) {
    // Log all MADT entries if debugging is enabled, but avoid unnecessary iteration in production
    #[cfg(debug_assertions)]
    {
        for madt_entry in madt.iter() {
            // Log the entry, but only if it's needed
            log::debug!("      {:#x?}", madt_entry);
        }
    }

    // Warn if MADT handling is not yet implemented on this platform
    #[cfg(debug_assertions)]
    {
        log::warn!("MADT not yet handled on this platform");
    }

    // In production, suppress unnecessary logging
    #[cfg(not(debug_assertions))]
    {
        // In production, log a simple warning about MADT not being handled, without iterating over all entries
        log::warn!("MADT handling is not yet implemented for this platform");
    }

}