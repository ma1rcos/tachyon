use core::{convert::TryFrom, mem, ptr};

use super::{rxsdt::Rxsdt, sdt::Sdt};

#[derive(Debug)]
pub struct Rsdt(&'static Sdt);

impl Rsdt {
    /// Initializes a new RSDT with secure validation
    pub fn new(sdt: &'static Sdt) -> Option<&'static Rsdt> {
        if sdt.signature == *b"RSDT" {
            Some(unsafe { &*(sdt as *const Sdt as *const Rsdt) })
        } else {
            None
        }
    }

    /// Returns the RSDT data as a safe byte slice
    pub fn as_slice(&self) -> Option<&[u8]> {
        let length = usize::try_from(self.0.length).ok()?;
        Some(unsafe { core::slice::from_raw_parts(self.0 as *const _ as *const u8, length) })
    }
}

impl Rxsdt for Rsdt {
    /// Iterates over RSDT inputs efficiently
    fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        RsdtIter { sdt: self.0, i: 0 }
    }
}

pub struct RsdtIter {
    sdt: &'static Sdt,
    i: usize,
}

impl Iterator for RsdtIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let data_len = self.sdt.data_len() / mem::size_of::<u32>();
        if self.i < data_len {
            let item = unsafe { ptr::read_unaligned((self.sdt.data_address() as *const u32).add(self.i)) };
            self.i += 1;
            Some(item as usize)
        } else {
            None
        }
    }
}