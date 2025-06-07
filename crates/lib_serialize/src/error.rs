// Copyright 2025 Velithris
// SPDX-License-Identifier: MIT

use core::fmt;
use std::error;

use core::num::ParseFloatError;
use weakauras_codec_lua_value::error::TryFromLuaValueError;

/// Errors than can occur while deserializing.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DeserializationError {
    /// The input does not start with a valid prefix.
    InvalidPrefix,
    /// Invalid tag.
    InvalidTag,
    /// Invalid embedded tag.
    InvalidEmbeddedTag,
    /// Invalid string reference.
    InvalidStringReference,
    /// Invalid map reference.
    InvalidMapReference,
    /// Failed to parse a floating-point number.
    InvalidFloatNumber,
    /// According to the input, a map has a key that is either a null or a NaN.
    /// That is not valid in Lua.
    InvalidMapKeyType,
    /// The input ended unexpectedly.
    UnexpectedEof,
    /// Exceeded recursion limit while deserializing nested data.
    RecursionLimitExceeded,
}

impl From<ParseFloatError> for DeserializationError {
    fn from(_value: ParseFloatError) -> Self {
        Self::InvalidFloatNumber
    }
}

impl From<TryFromLuaValueError> for DeserializationError {
    fn from(_value: TryFromLuaValueError) -> Self {
        Self::InvalidMapKeyType
    }
}

impl fmt::Display for DeserializationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidPrefix => write!(f, "Invalid prefix"),
            Self::InvalidTag => write!(f, "Invalid tag"),
            Self::InvalidEmbeddedTag => write!(f, "Invalid embedded tag"),
            Self::InvalidStringReference => write!(f, "Invalid string reference"),
            Self::InvalidMapReference => write!(f, "Invalid map reference"),
            Self::InvalidFloatNumber => write!(f, "Failed to parse a floating-point number"),
            Self::InvalidMapKeyType => write!(f, "Invalid map key type"),
            Self::UnexpectedEof => write!(f, "Unexpected EOF"),
            Self::RecursionLimitExceeded => write!(f, "Recursion limit exceeded"),
        }
    }
}

impl error::Error for DeserializationError {}

/// Errors than can occur while serializing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SerializationError {
    /// More than `2^24` unique strings.
    TooManyUniqueStrings,
    /// A string is larger than `2^24` bytes.
    StringIsTooLarge,
    /// A map is larger than `2^24` key-value pairs.
    MapIsTooLarge,
    /// An array is larger than `2^24` elements.
    ArrayIsTooLarge,
    /// Exceeded recursion limit while serializing nested data.
    RecursionLimitExceeded,
}

impl fmt::Display for SerializationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::TooManyUniqueStrings => write!(f, "Too many unique strings"),
            Self::StringIsTooLarge => write!(f, "String is too large"),
            Self::MapIsTooLarge => write!(f, "Map is too large"),
            Self::ArrayIsTooLarge => write!(f, "Array is too large"),
            Self::RecursionLimitExceeded => write!(f, "Recursion limit exceeded"),
        }
    }
}

impl error::Error for SerializationError {}
