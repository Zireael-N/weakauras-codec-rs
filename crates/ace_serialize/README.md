# weakauras-codec-ace-serialize

This library provides routines for deserializing and serializing [`LuaValues`]
in a way compatible with a Lua library called AceSerialize.

## Deserialization example

This is how you can use the library to deserialize strings produced by AceSerialize.

```rust
use weakauras_codec_ace_serialize::{DeserializationError, Deserializer};

fn main() -> Result<(), DeserializationError> {
    assert_eq!(
        Deserializer::from_str("^1^SHello,~`world!^^")
            .deserialize_first()?
            .unwrap(),
        "Hello, world!".into()
    );
    Ok(())
}
```

## Serialization example

This is how you can use the library to serialize values in a way compatible with AceSerialize.

```rust
use weakauras_codec_ace_serialize::{SerializationError, Serializer};

fn main() -> Result<(), SerializationError> {
    assert_eq!(
        Serializer::serialize_one(&"Hello, world!".into(), None)?,
        "^1^SHello,~`world!^^"
    );
    Ok(())
}
```

## Crate features

* **lua-value-fnv** - Use `fnv` instead of `BTreeMap` as the implementation of `LuaValue::Map`. **Disabled** by default.
* **lua-value-indexmap** - Use `indexmap` instead of `BTreeMap` as the implementation of `LuaValue::Map`. **Disabled** by default.
* **serde** - Allow serializing and deserializing `LuaValue` using `serde`. **Disabled** by default.

[`LuaValues`]: https://docs.rs/weakauras-codec-lua-value/latest/weakauras_codec_lua_value/enum.LuaValue.html
