use core::{mem, ptr};
use super::{find_sdt, sdt::Sdt};
use crate::{
    device::generic_timer::GenericTimer,
    dtb::irqchip::{register_irq, IRQ_CHIP},
};

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct Gtdt {
    pub header: Sdt,
    pub cnt_control_base: u64,
    _reserved: u32,
    pub secure_el1_timer_gsiv: u32,
    pub secure_el1_timer_flags: u32,
    pub non_secure_el1_timer_gsiv: u32,
    pub non_secure_el1_timer_flags: u32,
    pub virtual_el1_timer_gsiv: u32,
    pub virtual_el1_timer_flags: u32,
    pub el2_timer_gsiv: u32,
    pub el2_timer_flags: u32,
    pub cnt_read_base: u64,
    pub platform_timer_count: u32,
    pub platform_timer_offset: u32,
}

impl Gtdt {
    #[inline(always)]
    pub fn init() {
        let gtdt = Gtdt::new(find_sdt("GTDT").get(0)?)?;
        log::info!("generic_timer gsiv = {}", gtdt.non_secure_el1_timer_gsiv);

        let mut timer = GenericTimer {
            clk_freq: 0,
            reload_count: 0,
        };
        timer.init();

        register_irq(gtdt.non_secure_el1_timer_gsiv, timer);
        unsafe { IRQ_CHIP.irq_enable(gtdt.non_secure_el1_timer_gsiv) };
    }

    #[inline(always)]
    pub fn new(sdt: &'static Sdt) -> Option<&'static Gtdt> {
        (sdt.signature == *b"GTDT" && sdt.length as usize >= mem::size_of::<Gtdt>())
            .then(|| unsafe { &*ptr::cast::<_, Gtdt>(sdt) })
    }
}