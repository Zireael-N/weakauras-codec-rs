// Copyright 2025 Velithris
// SPDX-License-Identifier: MIT

use core::fmt;
use std::error;

/// The error type returned when conversion from [LuaValue] to [LuaMapKey] fails.
///
/// [LuaValue]: crate::LuaValue
/// [LuaMapKey]: crate::LuaMapKey
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TryFromLuaValueError {
    /// Attempt to convert a [LuaValue::Null](crate::LuaValue::Null) into a [LuaMapKey](crate::LuaMapKey).
    KeyCannotBeNan,
    /// Attempt to convert a [LuaValue::Number](crate::LuaValue::Number) that is NaN into a [LuaMapKey](crate::LuaMapKey).
    KeyCannotBeNull,
}

impl fmt::Display for TryFromLuaValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::KeyCannotBeNan => write!(
                f,
                "LuaMapKey can't be constructed from LuaValue::Number that is NaN"
            ),
            Self::KeyCannotBeNull => {
                write!(f, "LuaMapKey can't be constructed from LuaValue::Null")
            }
        }
    }
}

impl error::Error for TryFromLuaValueError {}
