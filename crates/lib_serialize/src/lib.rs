// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

pub mod deserialization;
pub mod error;
pub(crate) mod macros;
pub mod serialization;
pub(crate) mod type_tag;

pub use deserialization::Deserializer;
pub use error::*;
pub use serialization::Serializer;
pub(crate) use type_tag::{EmbeddedTypeTag, TypeTag};
pub use weakauras_codec_lua_value::LuaValue;

pub(crate) const FORMAT_VERSION: u8 = 1;
