// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

use std::borrow::Cow;

pub(crate) struct SliceReader<'s> {
    slice: &'s [u8],
    index: usize,
}

impl<'s> SliceReader<'s> {
    pub(crate) fn new(slice: &'s [u8]) -> Self {
        Self { slice, index: 0 }
    }

    #[inline]
    pub(crate) fn read_u8(&mut self) -> Option<u8> {
        if self.index < self.slice.len() {
            let byte = self.slice[self.index];
            self.index += 1;
            Some(byte)
        } else {
            None
        }
    }

    pub(crate) fn read_f64(&mut self) -> Option<f64> {
        if self.index + 7 < self.slice.len() {
            let mut buf = [0; 8];
            buf.copy_from_slice(&self.slice[self.index..self.index + 8]);
            self.index += 8;

            Some(f64::from_be_bytes(buf))
        } else {
            None
        }
    }

    pub(crate) fn read_int(&mut self, bytes: usize) -> Option<u64> {
        if (bytes > 0 && bytes <= 8) && (self.index + bytes - 1 < self.slice.len()) {
            let mut buf = [0; 8];
            buf[8 - bytes..].copy_from_slice(&self.slice[self.index..self.index + bytes]);
            self.index += bytes;

            Some(u64::from_be_bytes(buf))
        } else {
            None
        }
    }

    pub(crate) fn read_bytes(&mut self, len: usize) -> Option<&'s [u8]> {
        if self.index + len - 1 < self.slice.len() {
            let bytes = &self.slice[self.index..self.index + len];
            self.index += len;

            Some(bytes)
        } else {
            None
        }
    }

    pub(crate) fn read_string(&mut self, len: usize) -> Option<Cow<'s, str>> {
        if self.index + len - 1 < self.slice.len() {
            let s = String::from_utf8_lossy(&self.slice[self.index..self.index + len]);
            self.index += len;

            Some(s)
        } else {
            None
        }
    }
}
