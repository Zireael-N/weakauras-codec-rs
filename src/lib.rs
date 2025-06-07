// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

//! This library provides routines for decoding and encoding [WeakAuras]-compatible strings.
//!
//! # Decoding example
//!
//! This is how you can use the library to decode WeakAuras-compatible strings.
//!
//! ```
//! use weakauras_codec::{DecodeError, decode};
//!
//! fn main() -> Result<(), DecodeError> {
//!     let expected_value = "Hello, world!".into();
//!
//!     assert_eq!(
//!         decode(b"!lodJlypsnNCYxN6sO88lkNuumU4aaa", None)?.unwrap(),
//!         expected_value
//!     );
//!     assert_eq!(
//!         decode(b"!WA:2!JXl5rQ5Kt(6Oq55xuoPOiaa", Some(1024))?.unwrap(),
//!         expected_value
//!     );
//!
//!     Ok(())
//! }
//! ```
//!
//! # Encoding example
//!
//! This is how you can use the library to encode data as a WeakAuras-compatible string.
//!
//! ```
//! use std::error::Error;
//! use weakauras_codec::{OutputStringVersion, decode, encode};
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let value = "Hello, world!".into();
//!     let encoded_value_1 = encode(&value, OutputStringVersion::Deflate)?;
//!     let encoded_value_2 = encode(&value, OutputStringVersion::BinarySerialization)?;
//!
//!     assert_eq!(decode(encoded_value_1.as_bytes(), None)?.unwrap(), value);
//!     assert_eq!(decode(encoded_value_2.as_bytes(), None)?.unwrap(), value);
//!
//!     Ok(())
//! }
//! ```
//!
//! # Crate features
//!
//! * **legacy-strings-decoding** - Enable decoding of legacy WeakAuras-compatible strings. Uses a GPL-licensed library. **Disabled** by default.
//! * **gpl-dependencies** - Enable GPL-licensed dependencies. Currently, it enables the `legacy-strings-decoding` feature. **Disabled** by default.
//! * **flate2-rust-backend** - Enable the `rust-backend` feature in `flate2`. **Enabled** by default.
//! * **flate2-zlib-rs** - Enable the `zlib-rs` feature in `flate2`. **Disabled** by default.
//! * **flate2-zlib** - Enable the `zlib` feature in `flate2`. **Disabled** by default.
//! * **flate2-zlib-ng** - Enable the `zlib-ng` feature in `flate2`. **Disabled** by default.
//! * **flate2-zlib-ng-compat** - Enable the `zlib-ng-compat` feature in `flate2`. **Disabled** by default.
//! * **flate2-cloudflare-zlib** - Enable the `cloudflare_zlib` feature in `flate2`. **Disabled** by default.
//! * **lua-value-arbitrary** - Implement `arbitrary::Arbitrary` for [`LuaValue`]. **Disabled** by default.
//! * **lua-value-fnv** - Use `fnv` instead of `BTreeMap` as the implementation of [`LuaValue::Map`]. **Disabled** by default.
//! * **lua-value-indexmap** - Use `indexmap` instead of `BTreeMap` as the implementation of [`LuaValue::Map`]. **Disabled** by default.
//! * **serde** - Allow serializing and deserializing [`LuaValue`] using `serde`. **Disabled** by default.
//!
//! [WeakAuras]: https://weakauras.wtf

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Error types.
pub mod error;
pub use error::*;

use weakauras_codec_ace_serialize::{
    Deserializer as LegacyDeserializer, Serializer as LegacySerializer,
};
use weakauras_codec_base64::error::DecodeError as Base64DecodeError;
use weakauras_codec_lib_serialize::{Deserializer, Serializer};
pub use weakauras_codec_lua_value::LuaValue;

#[derive(Clone, Copy, PartialEq, Eq)]
enum StringVersion {
    #[cfg(feature = "legacy-strings-decoding")]
    Legacy, // base64
    Deflate,             // ! + base64
    BinarySerialization, // !WA:2! + base64
}

/// A version of the string to be produced by [encode].
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OutputStringVersion {
    /// `!` + base64-string
    Deflate,
    /// `!WA:2!` + base64-string
    BinarySerialization,
}

/// Decodes a WeakAuras-compatible string and returns a [LuaValue].
///
/// The second argument, `max_size`, is used as a counter-DoS measure. Since the data
/// is compressed, it's possible to construct a payload that would consume a lot of memory
/// after decompression. `None` is equivalent to 16 MiB.
///
/// # Example
///
/// ```
/// use weakauras_codec::{DecodeError, decode};
///
/// fn main() -> Result<(), DecodeError> {
///     let expected_value = "Hello, world!".into();
///
///     assert_eq!(
///         decode(b"!lodJlypsnNCYxN6sO88lkNuumU4aaa", None)?.unwrap(),
///         expected_value
///     );
///     assert_eq!(
///         decode(b"!WA:2!JXl5rQ5Kt(6Oq55xuoPOiaa", Some(1024))?.unwrap(),
///         expected_value
///     );
///
///     Ok(())
/// }
/// ```
pub fn decode(data: &[u8], max_size: Option<usize>) -> Result<Option<LuaValue>, DecodeError> {
    let (base64_data, version) = match data {
        [b'!', b'W', b'A', b':', b'2', b'!', rest @ ..] => {
            (rest, StringVersion::BinarySerialization)
        }
        [b'!', rest @ ..] => (rest, StringVersion::Deflate),
        _ => {
            #[cfg(feature = "legacy-strings-decoding")]
            {
                (data, StringVersion::Legacy)
            }

            #[cfg(not(feature = "legacy-strings-decoding"))]
            return Err(DecodeError::InvalidPrefix);
        }
    };

    let compressed_data = match weakauras_codec_base64::decode_to_vec(base64_data) {
        Ok(compressed_data) => compressed_data,
        Err(Base64DecodeError::InvalidByte(invalid_byte_at)) => {
            let prefix_len = base64_data.as_ptr().addr() - data.as_ptr().addr();

            return Err(DecodeError::Base64DecodeError(
                Base64DecodeError::InvalidByte(prefix_len + invalid_byte_at),
            ));
        }
        Err(e) => return Err(e.into()),
    };

    let max_size = max_size.unwrap_or(16 * 1024 * 1024);
    #[cfg(feature = "legacy-strings-decoding")]
    {
        if version == StringVersion::Legacy {
            let decoded = weakauras_codec_lib_compress::decompress(&compressed_data, max_size)?;
            return LegacyDeserializer::from_str(&String::from_utf8_lossy(&decoded))
                .deserialize_first()
                .map_err(Into::into);
        }
    }

    let decoded = {
        use flate2::read::DeflateDecoder;
        use std::io::prelude::*;

        let mut result = Vec::new();
        let mut inflater = DeflateDecoder::new(&compressed_data[..]).take(max_size as u64);

        inflater.read_to_end(&mut result)?;

        #[allow(clippy::unbuffered_bytes)] // inflater wraps in-memory data
        if result.len() == max_size && inflater.into_inner().bytes().next().is_some() {
            return Err(DecodeError::DataExceedsMaxSize);
        }

        result
    };

    Ok(if version == StringVersion::BinarySerialization {
        Deserializer::from_slice(&decoded).deserialize_first()?
    } else {
        LegacyDeserializer::from_str(&String::from_utf8_lossy(&decoded)).deserialize_first()?
    })
}

/// Encodes a [LuaValue] into a WeakAuras-compatible string.
///
/// # Example
///
/// ```
/// use std::error::Error;
/// use weakauras_codec::{OutputStringVersion, decode, encode};
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     let value = "Hello, world!".into();
///     let encoded_value_1 = encode(&value, OutputStringVersion::Deflate)?;
///     let encoded_value_2 = encode(&value, OutputStringVersion::BinarySerialization)?;
///
///     assert_eq!(decode(encoded_value_1.as_bytes(), None)?.unwrap(), value);
///     assert_eq!(decode(encoded_value_2.as_bytes(), None)?.unwrap(), value);
///
///     Ok(())
/// }
/// ```
pub fn encode(
    value: &LuaValue,
    string_version: OutputStringVersion,
) -> Result<String, EncodeError> {
    let (serialized, prefix) = match string_version {
        OutputStringVersion::Deflate => (
            LegacySerializer::serialize_one(value, None).map(|v| v.into_bytes())?,
            "!",
        ),
        OutputStringVersion::BinarySerialization => {
            (Serializer::serialize_one(value, None)?, "!WA:2!")
        }
    };

    let compressed = {
        use flate2::{Compression, read::DeflateEncoder};
        use std::io::prelude::*;

        let mut result = Vec::new();
        let mut deflater = DeflateEncoder::new(serialized.as_slice(), Compression::best());

        deflater.read_to_end(&mut result)?;
        result
    };

    Ok(weakauras_codec_base64::encode_to_string_with_prefix(
        &compressed,
        prefix,
    )?)
}
