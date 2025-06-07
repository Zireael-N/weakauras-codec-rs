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
    /// The input does not start with `^1`.
    InvalidPrefix,
    /// Invalid identifier. Valid values are `^^`, `^Z`, `^B`, `^b`, `^S`, `^N`, `^F`, `^f`, `^T`, `^t`.
    InvalidIdentifier,
    /// Invalid escape character. Valid ranges are `0x40..=0x5D`, `0x5E..=0x60` and `0x7A..=0x7D`.
    InvalidEscapeCharacter,
    /// Failed to parse a floating-point number.
    InvalidFloatNumber,
    /// A floating-point number stored as a mantissa-exponent pair is missing its exponent.
    MissingExponent,
    /// According to the input, a map has a key that is either a null or a NaN.
    /// That is not valid in Lua.
    InvalidMapKeyType,
    /// A map has a key without a corresponding value.
    MapMissingValue,
    /// The input ended before an identifier marking the end of a map.
    UnclosedMap,
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
            Self::InvalidIdentifier => write!(f, "Invalid identifier"),
            Self::InvalidEscapeCharacter => write!(f, "Invalid escape character"),
            Self::InvalidFloatNumber => write!(f, "Failed to parse a floating-point number"),
            Self::MissingExponent => write!(f, "A floating-point number is missing an exponent"),
            Self::InvalidMapKeyType => write!(f, "Invalid map key type"),
            Self::MapMissingValue => write!(f, "Map has a key without a corresponding value"),
            Self::UnclosedMap => write!(
                f,
                "Input ended before an identifier marking the end of a map"
            ),
            Self::UnexpectedEof => write!(f, "Unexpected EOF"),
            Self::RecursionLimitExceeded => write!(f, "Recursion limit exceeded"),
        }
    }
}

impl error::Error for DeserializationError {}

/// Errors than can occur while serializing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SerializationError {
    /// AceSerialize does not support serializing NaNs.
    NanEncountered,
    /// Exceeded recursion limit while serializing nested data.
    RecursionLimitExceeded,
}

impl fmt::Display for SerializationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NanEncountered => write!(f, "Encountered a NaN"),
            Self::RecursionLimitExceeded => write!(f, "Recursion limit exceeded"),
        }
    }
}

impl error::Error for SerializationError {}
