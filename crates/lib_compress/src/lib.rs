// Code extraction algorithm is based on code from LibCompress
// https://www.curseforge.com/wow/addons/libcompress
// Copyright 2008-2018 jjsheets, Galmok
// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: GPL-2.0-only

#![forbid(unsafe_code)]

mod bitfield;
pub mod error;
mod lookup_table;
mod utils;

pub use error::*;

use self::bitfield::Bitfield;
use lookup_table::{TableData, build_lookup_table};
use std::borrow::Cow;
use utils::{get_code, unescape_code};

pub fn decompress(input: &[u8], max_size: usize) -> Result<Cow<'_, [u8]>, DecompressionError> {
    let mut iter = input.iter();
    match iter.next() {
        Some(1) => return Ok(Cow::from(&input[1..])),
        Some(3) => {}
        _ => return Err(DecompressionError::InvalidPrefix),
    }

    let len = input.len();
    if len < 5 {
        return Err(DecompressionError::InputIsTooSmall);
    }

    let num_symbols = iter
        .next()
        .unwrap()
        .checked_add(1)
        .ok_or(DecompressionError::InvalidData)?;

    let original_size = iter
        .by_ref()
        .take(3)
        .map(|&byte| usize::from(byte))
        .enumerate()
        .fold(0, |acc, (i, byte)| acc + (byte << (i * 8)));

    if original_size == 0 {
        return Err(DecompressionError::InputIsTooSmall);
    }

    if original_size > max_size {
        return Err(DecompressionError::DataExceedsMaxSize);
    }

    let mut codes = Vec::with_capacity(num_symbols as usize);
    let mut result = Vec::with_capacity(original_size);

    let mut bitfield = Bitfield::new();

    let mut min_code_len = u8::MAX;
    let mut max_code_len = u8::MIN;

    // Code extraction:
    for _ in 0..num_symbols {
        let symbol = bitfield
            .insert_and_extract_byte(*iter.next().ok_or(DecompressionError::UnexpectedEof)?);

        loop {
            bitfield
                .insert(*iter.next().ok_or(DecompressionError::UnexpectedEof)?)
                .map_err(|_| DecompressionError::InvalidData)?;

            if let Some(v) = get_code(&mut bitfield)? {
                let (code, code_len) = unescape_code(v.0, v.1);
                min_code_len = core::cmp::min(min_code_len, code_len);
                max_code_len = core::cmp::max(max_code_len, code_len);

                codes.push((code, code_len, symbol));

                break;
            }
        }
    }
    codes.sort_unstable_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));

    // Decompression:
    let lut = build_lookup_table(&codes)?;

    loop {
        bitfield.fill_from_iterator(&mut iter);
        let original_len = bitfield.get_len();

        if bitfield.get_len() >= min_code_len {
            let mut cursor = &lut[(bitfield.peek_byte()) as usize];

            if bitfield.get_len() < cursor.code_length {
                break;
            }

            let mut new_bitfield = bitfield;
            while new_bitfield.get_len() >= cursor.code_length {
                if cursor.code_length == 0 {
                    return Err(DecompressionError::InvalidData);
                }

                match cursor.data {
                    TableData::Reference(ref v) => {
                        new_bitfield.discard_bits(cursor.code_length);
                        cursor = &v[(new_bitfield.peek_byte()) as usize];
                    }
                    TableData::Symbol(s) => {
                        result.push(s);
                        if result.len() == original_size {
                            return Ok(Cow::from(result));
                        }

                        bitfield = new_bitfield;
                        bitfield.discard_bits(cursor.code_length);
                        break;
                    }
                }
            }
        } else {
            break;
        }

        if bitfield.get_len() == original_len {
            return Err(DecompressionError::InvalidData);
        }
    }

    if result.len() == original_size {
        Ok(Cow::from(result))
    } else {
        Err(DecompressionError::InvalidData)
    }
}
