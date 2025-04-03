use alloc::{boxed::Box, vec::Vec};
use super::{Madt, MadtEntry};
use crate::{
    device::irqchip::{
        gic::{GenericInterruptController, GicCpuIf, GicDistIf},
        gicv3::{GicV3, GicV3CpuIf},
    },
    dtb::irqchip::{IrqChipItem, IRQ_CHIP},
    memory::{map_device_memory, PhysicalAddress, PAGE_SIZE},
};

/// Initializes the GIC (Generic Interrupt Controller) based on MADT table
pub(super) fn init(madt: &Madt) {
    let mut gicd_opt = None;
    let mut giccs = Vec::new();

    // Collect relevant MADT entries
    for madt_entry in madt.iter() {
        match madt_entry {
            MadtEntry::Gicc(gicc) => giccs.push(gicc),
            MadtEntry::Gicd(gicd) if gicd_opt.is_none() => gicd_opt = Some(gicd),
            MadtEntry::Gicd(_) => log::warn!("Multiple GICD entries found, ignoring extra ones"),
            _ => continue,
        }
    }

    let Some(gicd) = gicd_opt else {
        log::warn!("No GICD found, aborting initialization");
        return;
    };

    let mut gic_dist_if = GicDistIf::default();
    unsafe {
        let phys = PhysicalAddress::new(gicd.physical_base_address as usize);
        let virt = map_device_memory(phys, PAGE_SIZE);
        gic_dist_if.init(virt.data());
    }
    log::info!("Initialized GIC Distributor: {:#x?}", gic_dist_if);

    // Handle GIC versions separately
    match gicd.gic_version {
        1 | 2 => initialize_gic_v1_v2(&giccs, gic_dist_if),
        3 => initialize_gic_v3(&giccs, gic_dist_if),
        _ => log::warn!("Unsupported GIC version: {}", gicd.gic_version),
    }

    unsafe { IRQ_CHIP.init(None) };
}

/// Initializes GIC version 1 and 2
fn initialize_gic_v1_v2(giccs: &[&MadtGicc], gic_dist_if: GicDistIf) {
    for &gicc in giccs.iter().take(1) { // Only support the first GICC for now
        let mut gic_cpu_if = GicCpuIf::default();
        unsafe {
            let phys = PhysicalAddress::new(gicc.physical_base_address as usize);
            let virt = map_device_memory(phys, PAGE_SIZE);
            gic_cpu_if.init(virt.data());
        }
        log::info!("Initialized GIC CPU Interface: {:#x?}", gic_cpu_if);

        let gic = GenericInterruptController {
            gic_dist_if,
            gic_cpu_if,
            irq_range: (0, 0),
        };
        register_irq_chip(Box::new(gic));
    }
}

/// Initializes GIC version 3
fn initialize_gic_v3(giccs: &[&MadtGicc], gic_dist_if: GicDistIf) {
    for &gicc in giccs.iter().take(1) {
        let mut gic_cpu_if = GicV3CpuIf;
        unsafe { gic_cpu_if.init() };
        log::info!("Initialized GICv3 CPU Interface: {:#x?}", gic_cpu_if);

        let gic = GicV3 {
            gic_dist_if,
            gic_cpu_if,
            gicrs: Vec::new(), // TODO: Implement GIC Redistributors
            irq_range: (0, 0),
        };
        register_irq_chip(Box::new(gic));
    }
}

/// Registers an IRQ chip in the global IRQ chip list
fn register_irq_chip(chip: Box<dyn IrqChip>) {
    let irq_chip_item = IrqChipItem {
        phandle: 0,
        parents: Vec::new(),
        children: Vec::new(),
        ic: chip,
    };
    unsafe { IRQ_CHIP.irq_chip_list.chips.push(irq_chip_item) };
}