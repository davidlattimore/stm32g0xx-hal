//! # Cyclic redundancy check calculation unit
use crate::rcc::Rcc;
use crate::stm32::CRC;

pub enum InputReverse {
    None = 0b00,
    Byte = 0b01,
    HalfWord = 0b10,
    Word = 0b11,
}

pub enum PolySize {
    CRC32 = 0b00,
    CRC16 = 0b01,
    CRC8 = 0b10,
    CRC7 = 0b11,
}

pub struct Crc {
    rb: CRC,
}

impl Crc {
    pub fn polynomial(&mut self, size: PolySize, poly: u32) {
        self.reset();
        self.rb.pol.modify(|_, w| unsafe { w.bits(poly) });
        self.rb
            .cr
            .modify(|_, w| unsafe { w.polysize().bits(size as u8) });
    }

    pub fn seed(&mut self, value: u32) {
        self.rb.init.modify(|_, w| unsafe { w.bits(value) });
    }

    pub fn reverse_output(&mut self, rev_out: bool) {
        self.rb.cr.modify(|_, w| w.rev_out().bit(rev_out));
    }

    pub fn reverse_input(&mut self, rev_in: InputReverse) {
        self.rb
            .cr
            .modify(|_, w| unsafe { w.rev_in().bits(rev_in as u8) });
    }

    pub fn reset(&mut self) {
        self.rb.cr.modify(|_, w| w.reset().set_bit());
    }

    pub fn release(self) -> CRC {
        self.rb
    }
}

pub trait Digest<W> {
    fn digest(&mut self, data: W) -> u32;
}

impl Digest<u32> for Crc {
    fn digest(&mut self, data: u32) -> u32 {
        self.rb.dr.write(|w| unsafe { w.dr().bits(data) });
        self.rb.dr.read().bits()
    }
}

impl Digest<&[u32]> for Crc {
    fn digest(&mut self, data: &[u32]) -> u32 {
        data.iter().map(|v| self.digest(*v)).last();
        self.rb.dr.read().bits()
    }
}

impl Digest<&[u16]> for Crc {
    fn digest(&mut self, data: &[u16]) -> u32 {
        data.iter()
            .map(|v| unsafe {
                core::ptr::write_volatile(&self.rb.dr as *const _ as *mut u16, *v);
            })
            .last();
        self.rb.dr.read().bits()
    }
}

impl Digest<&[u8]> for Crc {
    fn digest(&mut self, data: &[u8]) -> u32 {
        let mut iter32 = data.chunks_exact(4);
        loop {
            match iter32.next() {
                Some(x) => {
                    self.digest(u32::from_be_bytes([x[0], x[1], x[2], x[3]]));
                },
                None => break,
            }
        }
        let mut iter16 = iter32.remainder().chunks_exact(2);
        loop {
            match iter16.next() {
                Some(x) => {
                    self.digest(&[u16::from_be_bytes([x[0], x[1]])][..]);
                },
                None => break,
            }
        }
        let mut iter8 = iter16.remainder().iter();
        loop {
            match iter8.next() {
                Some(x) => {
                    unsafe { core::ptr::write_volatile(&self.rb.dr as *const _ as *mut u8, *x); }
                },
                None => break,
            }
        }

        self.rb.dr.read().bits()
    }
}

impl Digest<&str> for Crc {
    fn digest(&mut self, s: &str) -> u32 {
        self.digest(&s.as_bytes()[..])
    }
}

pub trait CrcExt {
    fn constrain(self, rcc: &mut Rcc) -> Crc;
}

impl CrcExt for CRC {
    fn constrain(self, rcc: &mut Rcc) -> Crc {
        rcc.rb.ahbenr.modify(|_, w| w.crcen().set_bit());
        rcc.rb.ahbrstr.modify(|_, w| w.crcrst().set_bit());
        rcc.rb.ahbrstr.modify(|_, w| w.crcrst().clear_bit());
        Crc { rb: self }
    }
}
