// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

mod reader;

use crate::{error::DeserializationError, macros::check_recursion};
use reader::StrReader;
use weakauras_codec_lua_value::{LuaMapKey, LuaValue};

/// A structure for deserializing strings produced by AceSerialize.
///
/// # Example
///
/// ```
/// use weakauras_codec_ace_serialize::{deserialization::Deserializer, error::DeserializationError};
///
/// fn main() -> Result<(), DeserializationError> {
///     assert_eq!(
///         Deserializer::from_str("^1^SHello,~`world!^^")
///             .deserialize_first()?
///             .unwrap(),
///         "Hello, world!".into()
///     );
///     Ok(())
/// }
/// ```
pub struct Deserializer<'s> {
    remaining_depth: usize,
    reader: StrReader<'s>,
}

impl<'s> Deserializer<'s> {
    /// Create a deserializer from a string slice.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(slice: &'s str) -> Self {
        Self {
            remaining_depth: 128,
            reader: StrReader::new(slice),
        }
    }

    /// Deserialize all values.
    pub fn deserialize_all(mut self) -> Result<Vec<LuaValue>, DeserializationError> {
        self.reader.read_identifier().and_then(|v| {
            if v == "^1" {
                Ok(())
            } else {
                Err(DeserializationError::InvalidPrefix)
            }
        })?;

        let mut result = Vec::new();

        while self.reader.peek_identifier().is_ok() {
            if let Some(v) = self.deserialize_helper()? {
                result.push(v);
            }
        }

        Ok(result)
    }

    /// Deserialize the first value.
    pub fn deserialize_first(mut self) -> Result<Option<LuaValue>, DeserializationError> {
        self.reader.read_identifier().and_then(|v| {
            if v == "^1" {
                Ok(())
            } else {
                Err(DeserializationError::InvalidPrefix)
            }
        })?;

        self.deserialize_helper()
    }

    fn deserialize_helper(&mut self) -> Result<Option<LuaValue>, DeserializationError> {
        Ok(Some(match self.reader.read_identifier()? {
            "^^" => return Ok(None),
            "^Z" => LuaValue::Null,
            "^B" => LuaValue::Boolean(true),
            "^b" => LuaValue::Boolean(false),
            "^S" => LuaValue::String(String::from(self.reader.parse_str()?)),
            "^N" => LuaValue::Number(
                self.reader
                    .read_until_next()
                    .and_then(Self::deserialize_number)?,
            ),
            "^F" => {
                let mantissa = self
                    .reader
                    .read_until_next()
                    .and_then(|v| v.parse::<f64>().map_err(Into::into))?;
                let exponent = match self.reader.read_identifier()? {
                    "^f" => self
                        .reader
                        .read_until_next()
                        .and_then(|v| v.parse::<f64>().map_err(Into::into))?,
                    _ => return Err(DeserializationError::MissingExponent),
                };

                LuaValue::Number(mantissa * (2f64.powf(exponent)))
            }
            "^T" => {
                let mut keys = Vec::with_capacity(16);
                let mut values = Vec::with_capacity(16);
                loop {
                    match self.reader.peek_identifier()? {
                        "^t" => {
                            let _ = self.reader.read_identifier();
                            break;
                        }
                        _ => {
                            check_recursion!(self, DeserializationError, {
                                let key = LuaMapKey::try_from(
                                    self.deserialize_helper()?
                                        .ok_or(DeserializationError::UnclosedMap)?,
                                )?;

                                let value = match self.reader.peek_identifier()? {
                                    "^t" => {
                                        return Err(DeserializationError::MapMissingValue);
                                    }
                                    _ => self
                                        .deserialize_helper()?
                                        .ok_or(DeserializationError::UnclosedMap)?,
                                };
                                keys.push(key);
                                values.push(value);
                            });
                        }
                    }
                }

                debug_assert_eq!(keys.len(), values.len());
                let is_array = keys.iter().enumerate().all(|(i, key)| {
                    if let LuaValue::Number(key) = key.as_value() {
                        *key == (i + 1) as f64
                    } else {
                        false
                    }
                });

                if is_array {
                    LuaValue::Array(values)
                } else {
                    LuaValue::Map(keys.into_iter().zip(values).collect())
                }
            }
            _ => return Err(DeserializationError::InvalidIdentifier),
        }))
    }

    fn deserialize_number(data: &str) -> Result<f64, DeserializationError> {
        match data {
            "1.#INF" | "inf" => Ok(f64::INFINITY),
            "-1.#INF" | "-inf" => Ok(f64::NEG_INFINITY),
            v => v.parse().map_err(Into::into),
        }
    }
}
