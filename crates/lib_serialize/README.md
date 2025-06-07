# weakauras-codec-lib-serialize

This library provides routines for deserializing and serializing [`LuaValues`]
in a way compatible with a Lua library called LibSerialize.

## Deserialization example

This is how you can use the library to deserialize strings produced by LibSerialize.

```rust
use weakauras_codec_lib_serialize::{DeserializationError, Deserializer};

fn main() -> Result<(), DeserializationError> {
    assert_eq!(
        Deserializer::from_slice(b"\x01\xd2Hello, world!")
            .deserialize_first()?
            .unwrap(),
        "Hello, world!".into()
    );
    Ok(())
}
```

## Serialization example

This is how you can use the library to serialize values in a way compatible with LibSerialize.

```rust
use weakauras_codec_lib_serialize::{SerializationError, Serializer};

fn main() -> Result<(), SerializationError> {
    assert_eq!(
        Serializer::serialize_one(&"Hello, world!".into(), None)?,
        b"\x01\xd2Hello, world!"
    );
    Ok(())
}
```

## Crate features

* **lua-value-arbitrary** - Implement `arbitrary::Arbitrary` for `LuaValue`. **Disabled** by default.
* **lua-value-fnv** - Use `fnv` instead of `BTreeMap` as the implementation of `LuaValue::Map`. **Disabled** by default.
* **lua-value-indexmap** - Use `indexmap` instead of `BTreeMap` as the implementation of `LuaValue::Map`. **Disabled** by default.
* **serde** - Allow serializing and deserializing `LuaValue` using `serde`. **Disabled** by default.

[`LuaValues`]: https://docs.rs/weakauras-codec-lua-value/latest/weakauras_codec_lua_value/enum.LuaValue.html
