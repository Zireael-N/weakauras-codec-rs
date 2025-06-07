# weakauras-codec-lua-value

This library provides types that behave similarly to Lua types.

## Crate features

* **fnv** - Use `fnv` instead of `BTreeMap` as the implementation of `LuaValue::Map`. **Disabled** by default.
* **indexmap** - Use `indexmap` instead of `BTreeMap` as the implementation of `LuaValue::Map`. **Disabled** by default.
* **serde** - Allow serializing and deserializing `LuaValue` using `serde`. **Disabled** by default.
