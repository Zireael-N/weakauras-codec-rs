// Copyright 2025 Velithris
// SPDX-License-Identifier: MIT

use core::fmt;
use std::{error, io::Error as IoError};

use weakauras_codec_ace_serialize::error::{
    DeserializationError as AceSerializeDeserializationError,
    SerializationError as AceSerializeSerializationError,
};
use weakauras_codec_base64::error::{
    DecodeError as Base64DecodeError, EncodeError as Base64EncodeError,
};
#[cfg(feature = "legacy-strings-decoding")]
use weakauras_codec_lib_compress::error::DecompressionError as LibCompressDecompressionError;
use weakauras_codec_lib_serialize::error::{
    DeserializationError as LibSerializeDeserializationError,
    SerializationError as LibSerializeSerializationError,
};

/// Errors than can occur while decoding.
#[derive(Debug)]
#[non_exhaustive]
pub enum DecodeError {
    #[cfg(not(feature = "legacy-strings-decoding"))]
    /// The input does not start with a valid prefix.
    InvalidPrefix,
    /// The input is not a valid base64-string.
    Base64DecodeError(Base64DecodeError),
    #[cfg(feature = "legacy-strings-decoding")]
    /// The input is not valid data compressed by LibCompress.
    LibCompressDecompressionError(LibCompressDecompressionError),
    /// An [io::Error](std::io::Error) occurred while decompressing supposedly DEFLATE-compressed data.
    IoError(IoError),
    /// The compressed data exceeds provided maximum size.
    DataExceedsMaxSize,
    /// The input is not valid data serialized by AceSerialize.
    AceSerializeDeserializationError(AceSerializeDeserializationError),
    /// The input is not valid data serialized by LibSerialize.
    LibSerializeDeserializationError(LibSerializeDeserializationError),
}

impl From<Base64DecodeError> for DecodeError {
    fn from(value: Base64DecodeError) -> Self {
        Self::Base64DecodeError(value)
    }
}

#[cfg(feature = "legacy-strings-decoding")]
impl From<LibCompressDecompressionError> for DecodeError {
    fn from(value: LibCompressDecompressionError) -> Self {
        match value {
            LibCompressDecompressionError::DataExceedsMaxSize => Self::DataExceedsMaxSize,
            _ => Self::LibCompressDecompressionError(value),
        }
    }
}

impl From<IoError> for DecodeError {
    fn from(value: IoError) -> Self {
        Self::IoError(value)
    }
}

impl From<AceSerializeDeserializationError> for DecodeError {
    fn from(value: AceSerializeDeserializationError) -> Self {
        Self::AceSerializeDeserializationError(value)
    }
}

impl From<LibSerializeDeserializationError> for DecodeError {
    fn from(value: LibSerializeDeserializationError) -> Self {
        Self::LibSerializeDeserializationError(value)
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            #[cfg(not(feature = "legacy-strings-decoding"))]
            Self::InvalidPrefix => write!(f, "Invalid prefix"),
            Self::Base64DecodeError(inner) => write!(f, "Failed to decode base64: {}", inner),
            #[cfg(feature = "legacy-strings-decoding")]
            Self::LibCompressDecompressionError(inner) => {
                write!(f, "Failed to decompress data: {}", inner)
            }
            Self::IoError(inner) => write!(f, "Failed to decompress data: {}", inner),
            Self::DataExceedsMaxSize => write!(f, "Compressed data exceeds max size"),
            Self::AceSerializeDeserializationError(inner) => {
                write!(f, "Failed to deserialize data: {}", inner)
            }
            Self::LibSerializeDeserializationError(inner) => {
                write!(f, "Failed to deserialize data: {}", inner)
            }
        }
    }
}

impl error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            #[cfg(not(feature = "legacy-strings-decoding"))]
            Self::InvalidPrefix => None,
            Self::Base64DecodeError(inner) => Some(inner),
            #[cfg(feature = "legacy-strings-decoding")]
            Self::LibCompressDecompressionError(inner) => Some(inner),
            Self::IoError(inner) => Some(inner),
            Self::DataExceedsMaxSize => None,
            Self::AceSerializeDeserializationError(inner) => Some(inner),
            Self::LibSerializeDeserializationError(inner) => Some(inner),
        }
    }
}

/// Errors than can occur while encoding.
#[derive(Debug)]
#[non_exhaustive]
pub enum EncodeError {
    /// The input cannot be base64-encoded.
    Base64EncodeError(Base64EncodeError),
    /// An [io::Error](std::io::Error) occurred while compressing using DEFLATE.
    IoError(IoError),
    /// The input cannot be serialized using AceSerialize.
    AceSerializeSerializationError(AceSerializeSerializationError),
    /// The input cannot be serialized using LibSerialize.
    LibSerializeSerializationError(LibSerializeSerializationError),
}

impl From<Base64EncodeError> for EncodeError {
    fn from(value: Base64EncodeError) -> Self {
        Self::Base64EncodeError(value)
    }
}

impl From<IoError> for EncodeError {
    fn from(value: IoError) -> Self {
        Self::IoError(value)
    }
}

impl From<AceSerializeSerializationError> for EncodeError {
    fn from(value: AceSerializeSerializationError) -> Self {
        Self::AceSerializeSerializationError(value)
    }
}

impl From<LibSerializeSerializationError> for EncodeError {
    fn from(value: LibSerializeSerializationError) -> Self {
        Self::LibSerializeSerializationError(value)
    }
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Base64EncodeError(inner) => {
                write!(f, "Failed to encode data as base64: {}", inner)
            }
            Self::IoError(inner) => write!(f, "Failed to compress data: {}", inner),
            Self::AceSerializeSerializationError(inner) => {
                write!(f, "Failed to serialize data: {}", inner)
            }
            Self::LibSerializeSerializationError(inner) => {
                write!(f, "Failed to serialize data: {}", inner)
            }
        }
    }
}

impl error::Error for EncodeError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Base64EncodeError(inner) => Some(inner),
            Self::IoError(inner) => Some(inner),
            Self::AceSerializeSerializationError(inner) => Some(inner),
            Self::LibSerializeSerializationError(inner) => Some(inner),
        }
    }
}
