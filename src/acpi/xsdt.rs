use alloc::boxed::Box;
use core::{convert::TryFrom, mem, slice, ptr};

use super::{rxsdt::Rxsdt, sdt::Sdt};

#[derive(Debug)]
pub struct Xsdt(&'static Sdt);

impl Xsdt {
    /// Creates a new `Xsdt` instance if the given `Sdt` is a valid XSDT table.
    #[inline]
    pub fn new(sdt: &'static Sdt) -> Option<Self> {
        (sdt.signature == *b"XSDT").then(|| Xsdt(sdt))
    }

    /// Returns a slice representing the raw bytes of the XSDT.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        let length = usize::try_from(self.0.length).expect("32-bit length must fit in usize");
        unsafe { slice::from_raw_parts(self.0 as *const _ as *const u8, length) }
    }
}

impl Rxsdt for Xsdt {
    /// Returns an iterator over the XSDT entries.
    #[inline]
    fn iter(&self) -> Box<dyn Iterator<Item = usize> + '_> {
        Box::new(XsdtIter { sdt: self.0, i: 0 })
    }
}

pub struct XsdtIter {
    sdt: &'static Sdt,
    i: usize,
}

impl Iterator for XsdtIter {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let entry_count = self.sdt.data_len() / mem::size_of::<u64>();
        if self.i < entry_count {
            let ptr = (self.sdt.data_address() as *const u64).wrapping_add(self.i);
            let item = unsafe { ptr::read_unaligned(ptr) } as usize;
            self.i += 1;
            Some(item)
        } else {
            None
        }
    }
}