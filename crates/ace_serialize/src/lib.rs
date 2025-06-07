// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

//! This library provides routines for deserializing and serializing [`LuaValues`](LuaValue)
//! in a way compatible with a Lua library called AceSerialize.
//!
//! # Deserialization example
//!
//! This is how you can use the library to deserialize strings produced by AceSerialize.
//!
//! ```
//! use weakauras_codec_ace_serialize::{DeserializationError, Deserializer};
//!
//! fn main() -> Result<(), DeserializationError> {
//!     assert_eq!(
//!         Deserializer::from_str("^1^SHello,~`world!^^")
//!             .deserialize_first()?
//!             .unwrap(),
//!         "Hello, world!".into()
//!     );
//!     Ok(())
//! }
//! ```
//!
//! # Serialization example
//!
//! This is how you can use the library to serialize values in a way compatible with AceSerialize.
//!
//! ```
//! use weakauras_codec_ace_serialize::{SerializationError, Serializer};
//!
//! fn main() -> Result<(), SerializationError> {
//!     assert_eq!(
//!         Serializer::serialize_one(&"Hello, world!".into(), None)?,
//!         "^1^SHello,~`world!^^"
//!     );
//!     Ok(())
//! }
//! ```
//!
//! # Crate features
//!
//! * **lua-value-fnv** - Use `fnv` instead of `BTreeMap` as the implementation of [`LuaValue::Map`]. **Disabled** by default.
//! * **lua-value-indexmap** - Use `indexmap` instead of `BTreeMap` as the implementation of [`LuaValue::Map`]. **Disabled** by default.
//! * **serde** - Allow serializing and deserializing [`LuaValue`] using `serde`. **Disabled** by default.

#![deny(missing_docs)]

/// Deserialization.
pub mod deserialization;
/// Error types.
pub mod error;
pub(crate) mod macros;
/// Serialization.
pub mod serialization;

pub use deserialization::Deserializer;
pub use error::*;
pub use serialization::Serializer;
pub use weakauras_codec_lua_value::LuaValue;
