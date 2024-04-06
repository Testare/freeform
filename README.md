Freeform is a small library for being able to store free-form typed ser/de data, sort of like a specialized `HashMap<String, Box<Any>>`.

## Current implementation

Current implementation is pretty minimal, with a single `Freeform` type. You can store and retrieve values by string so 
long as the type of the values implement the serde `Serialize`/`Deserialize` traits. The values are serialized to store 
as a JSON string and deserialized when requested.

While this can be done with normal strings and generics, the recommended API is to use `typed_key` macro from the crate
of the same name. You can define a constant `Key` which has a string and a associated type, and use that constant when
storing or retrieving values from the `Freeform`.

## Future plans

In the future, there are a number of optimizations I would like to implement.

* Storing deserialized representations as well, and only serializing when needed
* Caching serialized/deserialized forms, and only generating them when required
* Support for more serde implementations (serde_json, toml, ron, etc.)