// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

pub mod deserialization;
pub mod error;
pub(crate) mod macros;
pub mod serialization;

pub use deserialization::Deserializer;
pub use error::*;
pub use serialization::Serializer;
pub use weakauras_codec_lua_value::LuaValue;
