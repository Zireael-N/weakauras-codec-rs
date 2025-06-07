// Copyright 2025 Velithris
// SPDX-License-Identifier: MIT

use core::fmt;
#[cfg(any(test, feature = "std"))]
use std::error;

/// Errors than can occur while decoding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecodeError {
    /// An invalid byte was found in the input. Its offset is provided.
    InvalidByte(usize),
    /// The input's length is invalid.
    InvalidLength,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::InvalidByte(offset) => write!(f, "Invalid byte at offset {}", offset),
            Self::InvalidLength => write!(f, "Invalid length"),
        }
    }
}

#[cfg(any(test, feature = "std"))]
impl error::Error for DecodeError {}

/// Errors than can occur while decoding into a slice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecodeIntoSliceError {
    /// A [DecodeError] occurred.
    DecodeError(DecodeError),
    /// The provided slice is too small.
    OutputSliceIsTooSmall,
}

impl From<DecodeError> for DecodeIntoSliceError {
    fn from(e: DecodeError) -> Self {
        Self::DecodeError(e)
    }
}

impl fmt::Display for DecodeIntoSliceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DecodeError(inner) => inner.fmt(f),
            Self::OutputSliceIsTooSmall => write!(f, "Output slice is too small"),
        }
    }
}

#[cfg(any(test, feature = "std"))]
impl error::Error for DecodeIntoSliceError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::DecodeError(inner) => Some(inner),
            Self::OutputSliceIsTooSmall => None,
        }
    }
}

/// Errors than can occur while encoding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EncodeError {
    /// Encoding the input would require more than [usize::MAX] bytes of output.
    DataIsTooLarge,
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DataIsTooLarge => write!(f, "Data is too large"),
        }
    }
}

#[cfg(any(test, feature = "std"))]
impl error::Error for EncodeError {}

/// Errors than can occur while encoding into a slice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EncodeIntoSliceError {
    /// An [EncodeError] occurred.
    EncodeError(EncodeError),
    /// The provided slice is too small.
    OutputSliceIsTooSmall,
}

impl From<EncodeError> for EncodeIntoSliceError {
    fn from(e: EncodeError) -> Self {
        Self::EncodeError(e)
    }
}

impl fmt::Display for EncodeIntoSliceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EncodeError(inner) => inner.fmt(f),
            Self::OutputSliceIsTooSmall => write!(f, "Output slice is too small"),
        }
    }
}

#[cfg(any(test, feature = "std"))]
impl error::Error for EncodeIntoSliceError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::EncodeError(inner) => Some(inner),
            Self::OutputSliceIsTooSmall => None,
        }
    }
}
