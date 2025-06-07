// Copyright 2025 Velithris
// SPDX-License-Identifier: GPL-2.0-or-later

use core::fmt;
use std::error;

/// Errors than can occur while decompressing.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DecompressionError {
    /// Invalid prefix.
    InvalidPrefix,
    /// The input is too small.
    InputIsTooSmall,
    /// Compressed data exceeds provided maximum size.
    DataExceedsMaxSize,
    /// The input ended unexpectedly.
    UnexpectedEof,
    /// Catch-all variant for bogus input.
    InvalidData,
}

impl fmt::Display for DecompressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidPrefix => write!(f, "Invalid prefix"),
            Self::InputIsTooSmall => write!(f, "Input is too small"),
            Self::DataExceedsMaxSize => write!(f, "Compressed data exceeds max size"),
            Self::UnexpectedEof => write!(f, "Unexpected EOF"),
            Self::InvalidData => write!(f, "Invalid data"),
        }
    }
}

impl error::Error for DecompressionError {}
