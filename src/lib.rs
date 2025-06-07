// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

#![forbid(unsafe_code)]

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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OutputStringVersion {
    /// `!` + base64-string
    Deflate,
    /// `!WA:2!` + base64-string
    BinarySerialization,
}

/// Takes a string encoded by WeakAuras and returns
/// a [LuaValue].
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

/// Takes a [LuaValue] and returns
/// a string that can be decoded by WeakAuras.
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
