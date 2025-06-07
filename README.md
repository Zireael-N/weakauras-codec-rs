# weakauras-codec

This library provides routines for decoding and encoding [WeakAuras]-compatible strings.

## Decoding example

This is how you can use the library to decode WeakAuras-compatible strings.

```rust
use weakauras_codec::{DecodeError, decode};

fn main() -> Result<(), DecodeError> {
    let expected_value = "Hello, world!".into();

    assert_eq!(
        decode(b"!lodJlypsnNCYxN6sO88lkNuumU4aaa", None)?.unwrap(),
        expected_value
    );
    assert_eq!(
        decode(b"!WA:2!JXl5rQ5Kt(6Oq55xuoPOiaa", Some(1024))?.unwrap(),
        expected_value
    );

    Ok(())
}
```

## Encoding example

This is how you can use the library to encode data as a WeakAuras-compatible string.

```rust
use std::error::Error;
use weakauras_codec::{OutputStringVersion, decode, encode};

fn main() -> Result<(), Box<dyn Error>> {
    let value = "Hello, world!".into();
    let encoded_value_1 = encode(&value, OutputStringVersion::Deflate)?;
    let encoded_value_2 = encode(&value, OutputStringVersion::BinarySerialization)?;

    assert_eq!(decode(encoded_value_1.as_bytes(), None)?.unwrap(), value);
    assert_eq!(decode(encoded_value_2.as_bytes(), None)?.unwrap(), value);

    Ok(())
}
```

## Crate features

* **legacy-strings-decoding** - Enable decoding of legacy WeakAuras-compatible strings. Uses a GPL-licensed library. **Disabled** by default.
* **gpl-dependencies** - Enable GPL-licensed dependencies. Currently, it enables the `legacy-strings-decoding` feature. **Disabled** by default.
* **flate2-rust-backend** - Enable the `rust-backend` feature in `flate2`. **Enabled** by default.
* **flate2-zlib-rs** - Enable the `zlib-rs` feature in `flate2`. **Disabled** by default.
* **flate2-zlib** - Enable the `zlib` feature in `flate2`. **Disabled** by default.
* **flate2-zlib-ng** - Enable the `zlib-ng` feature in `flate2`. **Disabled** by default.
* **flate2-zlib-ng-compat** - Enable the `zlib-ng-compat` feature in `flate2`. **Disabled** by default.
* **flate2-cloudflare-zlib** - Enable the `cloudflare_zlib` feature in `flate2`. **Disabled** by default.
* **lua-value-fnv** - Use `fnv` instead of `BTreeMap` as the implementation of `LuaValue::Map`. **Disabled** by default.
* **lua-value-indexmap** - Use `indexmap` instead of `BTreeMap` as the implementation of `LuaValue::Map`. **Disabled** by default.
* **serde** - Allow serializing and deserializing `LuaValue` using `serde`. **Disabled** by default.

[WeakAuras]: https://weakauras.wtf
