// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

pub mod deserialization;
pub(crate) mod macros;
pub mod serialization;

pub use deserialization::Deserializer;
pub use serialization::Serializer;
pub use weakauras_codec_lua_value::LuaValue;
