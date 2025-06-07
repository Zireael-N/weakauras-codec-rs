// Based on SliceRead from serde_json
// https://github.com/serde-rs/json
// Copyright 2016-2020 David Tolnay
// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::error::DeserializationError;

pub(crate) struct StrReader<'s> {
    slice: &'s [u8],
    index: usize,
    scratch: Vec<u8>,
}

impl<'s> StrReader<'s> {
    pub(crate) fn new(slice: &'s str) -> Self {
        Self {
            slice: slice.as_bytes(),
            index: 0,
            scratch: Vec::new(),
        }
    }

    #[allow(dead_code)]
    #[inline]
    fn next(&mut self) -> Option<u8> {
        if self.index < self.slice.len() {
            let c = self.slice[self.index];
            self.index += 1;
            Some(c)
        } else {
            None
        }
    }

    #[inline]
    fn peek(&self) -> Option<u8> {
        if self.index < self.slice.len() {
            Some(self.slice[self.index])
        } else {
            None
        }
    }

    #[inline]
    fn discard(&mut self) {
        self.index += 1;
    }

    #[allow(dead_code)]
    #[inline]
    pub(crate) fn position(&self) -> usize {
        self.index
    }

    pub(crate) fn read_identifier(&mut self) -> Result<&str, DeserializationError> {
        if self.index + 1 < self.slice.len() {
            // Matching against the second byte to ensure
            // we don't break up a multibyte character.
            match (self.slice[self.index], self.slice[self.index + 1]) {
                (b'^', 0x00..=0x79) => {
                    let result = unsafe {
                        core::str::from_utf8_unchecked(&self.slice[self.index..self.index + 2])
                    };
                    self.index += 2;
                    Ok(result)
                }
                _ => Err(DeserializationError::InvalidIdentifier),
            }
        } else {
            Err(DeserializationError::UnexpectedEof)
        }
    }

    pub(crate) fn peek_identifier(&self) -> Result<&str, DeserializationError> {
        if self.index + 1 < self.slice.len() {
            // Matching against the second byte to ensure
            // we don't break up a multibyte character.
            match (self.slice[self.index], self.slice[self.index + 1]) {
                (b'^', 0x00..=0x79) => Ok(unsafe {
                    core::str::from_utf8_unchecked(&self.slice[self.index..self.index + 2])
                }),
                _ => Err(DeserializationError::InvalidIdentifier),
            }
        } else {
            Err(DeserializationError::UnexpectedEof)
        }
    }

    pub(crate) fn read_until_next(&mut self) -> Result<&str, DeserializationError> {
        let start = self.index;

        loop {
            match self.peek() {
                None => return Err(DeserializationError::UnexpectedEof),
                Some(b'^') => {
                    // SAFETY: As long as `start` does not point at the middle
                    // of a multibyte character, this should be safe.
                    // Public API does not allow the reader to end up in such a state.
                    return Ok(unsafe {
                        core::str::from_utf8_unchecked(&self.slice[start..self.index])
                    });
                }
                _ => self.discard(),
            }
        }
    }

    pub(crate) fn parse_str(&mut self) -> Result<&str, DeserializationError> {
        self.scratch.clear();

        let mut copy_from = self.index;

        loop {
            match self.peek() {
                None => return Err(DeserializationError::UnexpectedEof),
                Some(b'^') => {
                    if self.scratch.is_empty() {
                        // SAFETY: As long as `copy_from` does not point at the middle
                        // of a multibyte character, this should be safe.
                        // Public API does not allow the reader to end up in such a state.
                        return Ok(unsafe {
                            core::str::from_utf8_unchecked(&self.slice[copy_from..self.index])
                        });
                    } else {
                        // SAFETY: None of the replaced bytes and their replacements
                        // has the most significant bit set to 1.
                        self.scratch
                            .extend_from_slice(&self.slice[copy_from..self.index]);
                        return Ok(unsafe { core::str::from_utf8_unchecked(&self.scratch) });
                    }
                }
                Some(b'~') => {
                    self.scratch
                        .extend_from_slice(&self.slice[copy_from..self.index]);

                    self.discard();

                    let replacement = match self.peek() {
                        Some(v @ 0x40..=0x5D) | Some(v @ 0x5F..=0x60) => v - 64,
                        Some(0x7A) => 0x1E,
                        Some(0x7B) => 0x7F,
                        Some(0x7C) => 0x7E,
                        Some(0x7D) => 0x5E,
                        _ => return Err(DeserializationError::InvalidEscapeCharacter),
                    };

                    self.discard();
                    self.scratch.push(replacement);

                    copy_from = self.index;
                }
                _ => self.discard(),
            }
        }
    }
}
