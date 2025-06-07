// Based on code from LibSerialize
// https://github.com/rossnichols/LibSerialize
// Copyright 2020-2021 Ross Nichols
// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

mod reader;

use crate::{
    EmbeddedTypeTag, FORMAT_VERSION, TypeTag, error::DeserializationError, macros::check_recursion,
};
use reader::SliceReader;
use weakauras_codec_lua_value::{LuaMapKey, LuaValue, Map};

pub struct Deserializer<'s> {
    remaining_depth: usize,
    reader: SliceReader<'s>,

    table_refs: Vec<LuaValue>,
    string_refs: Vec<String>,
}

impl<'s> Deserializer<'s> {
    pub fn from_slice(slice: &'s [u8]) -> Self {
        Self {
            remaining_depth: 128,
            reader: SliceReader::new(slice),

            table_refs: Vec::new(),
            string_refs: Vec::new(),
        }
    }

    /// Deserialize all values.
    pub fn deserialize_all(mut self) -> Result<Vec<LuaValue>, DeserializationError> {
        match self.reader.read_u8() {
            Some(val) if val == FORMAT_VERSION || val == FORMAT_VERSION + 1 => {}
            _ => return Err(DeserializationError::InvalidPrefix),
        }

        let mut result = Vec::new();

        while let Some(v) = self.deserialize_helper()? {
            result.push(v);
        }

        Ok(result)
    }

    /// Deserialize the first value.
    pub fn deserialize_first(mut self) -> Result<Option<LuaValue>, DeserializationError> {
        match self.reader.read_u8() {
            Some(val) if val == FORMAT_VERSION || val == FORMAT_VERSION + 1 => {}
            _ => return Err(DeserializationError::InvalidPrefix),
        }

        self.deserialize_helper()
    }

    fn deserialize_helper(&mut self) -> Result<Option<LuaValue>, DeserializationError> {
        match self.reader.read_u8() {
            None => Ok(None),
            Some(value) => {
                if value & 1 == 1 {
                    // `NNNN NNN1`: a 7 bit non-negative int
                    Ok(Some(LuaValue::Number((value >> 1) as f64)))
                } else if value & 3 == 2 {
                    // * `CCCC TT10`: a 2 bit type index and 4 bit count (strlen, #tab, etc.)
                    //     * Followed by the type-dependent payload
                    let tag = EmbeddedTypeTag::from_u8((value & 0x0F) >> 2)
                        .ok_or(DeserializationError::InvalidEmbeddedTag)?;
                    let len = value >> 4;

                    self.deserialize_embedded(tag, len).map(Option::Some)
                } else if value & 7 == 4 {
                    // * `NNNN S100`: the lower four bits of a 12 bit int and 1 bit for its sign
                    //     * Followed by a byte for the upper bits
                    let next_byte = self
                        .reader
                        .read_u8()
                        .ok_or(DeserializationError::UnexpectedEof)?
                        as u16;
                    let packed = (next_byte << 8) + value as u16;

                    Ok(Some(LuaValue::Number(if value & 15 == 12 {
                        -((packed >> 4) as f64)
                    } else {
                        (packed >> 4) as f64
                    })))
                } else {
                    // * `TTTT T000`: a 5 bit type index
                    //     * Followed by the type-dependent payload, including count(s) if needed
                    let tag =
                        TypeTag::from_u8(value >> 3).ok_or(DeserializationError::InvalidTag)?;

                    self.deserialize_one(tag).map(Option::Some)
                }
            }
        }
    }

    #[inline(always)]
    fn extract_value(&mut self) -> Result<LuaValue, DeserializationError> {
        match self.deserialize_helper() {
            Ok(Some(value)) => Ok(value),
            Ok(None) => Err(DeserializationError::UnexpectedEof),
            Err(e) => Err(e),
        }
    }

    fn deserialize_embedded(
        &mut self,
        tag: EmbeddedTypeTag,
        len: u8,
    ) -> Result<LuaValue, DeserializationError> {
        match tag {
            EmbeddedTypeTag::Str => self.deserialize_string(len as usize),
            EmbeddedTypeTag::Map => self.deserialize_map(len as usize),
            EmbeddedTypeTag::Array => self.deserialize_array(len as usize),
            // For MIXED, the 4-bit count contains two 2-bit counts that are one less than the true count.
            EmbeddedTypeTag::Mixed => {
                self.deserialize_mixed(((len & 3) + 1) as usize, ((len >> 2) + 1) as usize)
            }
        }
    }

    fn deserialize_one(&mut self, tag: TypeTag) -> Result<LuaValue, DeserializationError> {
        match tag {
            TypeTag::Null => Ok(LuaValue::Null),

            TypeTag::Int16Pos => self.deserialize_int(2).map(|v| LuaValue::Number(v as f64)),
            TypeTag::Int16Neg => self
                .deserialize_int(2)
                .map(|v| LuaValue::Number(-(v as f64))),
            TypeTag::Int24Pos => self.deserialize_int(3).map(|v| LuaValue::Number(v as f64)),
            TypeTag::Int24Neg => self
                .deserialize_int(3)
                .map(|v| LuaValue::Number(-(v as f64))),
            TypeTag::Int32Pos => self.deserialize_int(4).map(|v| LuaValue::Number(v as f64)),
            TypeTag::Int32Neg => self
                .deserialize_int(4)
                .map(|v| LuaValue::Number(-(v as f64))),
            TypeTag::Int64Pos => self.deserialize_int(7).map(|v| LuaValue::Number(v as f64)),
            TypeTag::Int64Neg => self
                .deserialize_int(7)
                .map(|v| LuaValue::Number(-(v as f64))),

            TypeTag::Float => self.deserialize_f64().map(LuaValue::Number),
            TypeTag::FloatStrPos => self.deserialize_f64_from_str().map(LuaValue::Number),
            TypeTag::FloatStrNeg => self
                .deserialize_f64_from_str()
                .map(|v| LuaValue::Number(-v)),

            TypeTag::True => Ok(LuaValue::Boolean(true)),
            TypeTag::False => Ok(LuaValue::Boolean(false)),

            TypeTag::Str8 => {
                let len = self
                    .reader
                    .read_u8()
                    .ok_or(DeserializationError::UnexpectedEof)?;
                self.deserialize_string(len as usize)
            }
            TypeTag::Str16 => {
                let len = self.deserialize_int(2)?;
                self.deserialize_string(len as usize)
            }
            TypeTag::Str24 => {
                let len = self.deserialize_int(3)?;
                self.deserialize_string(len as usize)
            }

            TypeTag::Map8 => {
                let len = self
                    .reader
                    .read_u8()
                    .ok_or(DeserializationError::UnexpectedEof)?;
                self.deserialize_map(len as usize)
            }
            TypeTag::Map16 => {
                let len = self.deserialize_int(2)?;
                self.deserialize_map(len as usize)
            }
            TypeTag::Map24 => {
                let len = self.deserialize_int(3)?;
                self.deserialize_map(len as usize)
            }

            TypeTag::Array8 => {
                let len = self
                    .reader
                    .read_u8()
                    .ok_or(DeserializationError::UnexpectedEof)?;
                self.deserialize_array(len as usize)
            }
            TypeTag::Array16 => {
                let len = self.deserialize_int(2)?;
                self.deserialize_array(len as usize)
            }
            TypeTag::Array24 => {
                let len = self.deserialize_int(3)?;
                self.deserialize_array(len as usize)
            }

            TypeTag::Mixed8 => {
                let array_len = self
                    .reader
                    .read_u8()
                    .ok_or(DeserializationError::UnexpectedEof)?;
                let map_len = self
                    .reader
                    .read_u8()
                    .ok_or(DeserializationError::UnexpectedEof)?;

                self.deserialize_mixed(array_len as usize, map_len as usize)
            }
            TypeTag::Mixed16 => {
                let array_len = self.deserialize_int(2)?;
                let map_len = self.deserialize_int(2)?;

                self.deserialize_mixed(array_len as usize, map_len as usize)
            }
            TypeTag::Mixed24 => {
                let array_len = self.deserialize_int(3)?;
                let map_len = self.deserialize_int(3)?;

                self.deserialize_mixed(array_len as usize, map_len as usize)
            }

            TypeTag::StrRef8 => {
                let index = self
                    .reader
                    .read_u8()
                    .ok_or(DeserializationError::UnexpectedEof)?
                    - 1;
                match self.string_refs.get(index as usize) {
                    None => Err(DeserializationError::InvalidStringReference),
                    Some(s) => Ok(LuaValue::String(s.clone())),
                }
            }
            TypeTag::StrRef16 => {
                let index = self.deserialize_int(2)? - 1;
                match self.string_refs.get(index as usize) {
                    None => Err(DeserializationError::InvalidStringReference),
                    Some(s) => Ok(LuaValue::String(s.clone())),
                }
            }
            TypeTag::StrRef24 => {
                let index = self.deserialize_int(3)? - 1;
                match self.string_refs.get(index as usize) {
                    None => Err(DeserializationError::InvalidStringReference),
                    Some(s) => Ok(LuaValue::String(s.clone())),
                }
            }

            TypeTag::MapRef8 => {
                let index = self
                    .reader
                    .read_u8()
                    .ok_or(DeserializationError::UnexpectedEof)?
                    - 1;
                match self.table_refs.get(index as usize) {
                    None => Err(DeserializationError::InvalidMapReference),
                    Some(v) => Ok(v.clone()),
                }
            }
            TypeTag::MapRef16 => {
                let index = self.deserialize_int(2)? - 1;
                match self.table_refs.get(index as usize) {
                    None => Err(DeserializationError::InvalidMapReference),
                    Some(v) => Ok(v.clone()),
                }
            }
            TypeTag::MapRef24 => {
                let index = self.deserialize_int(3)? - 1;
                match self.table_refs.get(index as usize) {
                    None => Err(DeserializationError::InvalidMapReference),
                    Some(v) => Ok(v.clone()),
                }
            }
        }
    }

    fn deserialize_string(&mut self, len: usize) -> Result<LuaValue, DeserializationError> {
        match self.reader.read_string(len) {
            None => Err(DeserializationError::UnexpectedEof),
            Some(s) => {
                let s = s.into_owned();
                if len > 2 {
                    self.string_refs.push(s.clone());
                }

                Ok(LuaValue::String(s))
            }
        }
    }

    fn deserialize_f64(&mut self) -> Result<f64, DeserializationError> {
        match self.reader.read_f64() {
            None => Err(DeserializationError::UnexpectedEof),
            Some(v) => Ok(v),
        }
    }

    fn deserialize_f64_from_str(&mut self) -> Result<f64, DeserializationError> {
        let len = self
            .reader
            .read_u8()
            .ok_or(DeserializationError::UnexpectedEof)?;

        match self.reader.read_bytes(len as usize) {
            None => Err(DeserializationError::UnexpectedEof),
            Some(bytes) => core::str::from_utf8(bytes)
                .ok()
                .and_then(|s| s.parse::<f64>().ok())
                .ok_or(DeserializationError::InvalidFloatNumber),
        }
    }

    fn deserialize_int(&mut self, bytes: usize) -> Result<u64, DeserializationError> {
        match self.reader.read_int(bytes) {
            None => Err(DeserializationError::UnexpectedEof),
            Some(v) => Ok(v),
        }
    }

    fn deserialize_map(&mut self, len: usize) -> Result<LuaValue, DeserializationError> {
        let mut m = Map::new();

        for _ in 0..len {
            check_recursion!(self, DeserializationError, {
                let (key, value) = (self.extract_value()?, self.extract_value()?);

                m.insert(LuaMapKey::try_from(key)?, value);
            });
        }

        let m = LuaValue::Map(m);
        self.table_refs.push(m.clone());
        Ok(m)
    }

    fn deserialize_array(&mut self, len: usize) -> Result<LuaValue, DeserializationError> {
        let mut v = Vec::new();

        for _ in 0..len {
            check_recursion!(self, DeserializationError, {
                v.push(self.extract_value()?);
            });
        }

        let v = LuaValue::Array(v);
        self.table_refs.push(v.clone());
        Ok(v)
    }

    fn deserialize_mixed(
        &mut self,
        array_len: usize,
        map_len: usize,
    ) -> Result<LuaValue, DeserializationError> {
        let mut m = Map::new();

        for i in 1..=array_len {
            check_recursion!(self, DeserializationError, {
                let el = self.extract_value()?;
                m.insert(LuaMapKey::try_from(LuaValue::Number(i as f64)).unwrap(), el);
            });
        }

        for _ in 0..map_len {
            check_recursion!(self, DeserializationError, {
                let (key, value) = (self.extract_value()?, self.extract_value()?);

                m.insert(LuaMapKey::try_from(key)?, value);
            });
        }

        let m = LuaValue::Map(m);
        self.table_refs.push(m.clone());
        Ok(m)
    }
}
