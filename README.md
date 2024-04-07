Freeform is a small library for being able to store free-form typed ser/de data, sort of like a specialized `HashMap<String, Box<dyn Any>>` for
data types that are commonly serialized/deserialized.

## Current implementation

The main dish of this crate is the `Freeform` type. You can store and retrieve values using `Key<T>`'s from the 
`typed_key` crate, so long as `T` implements `FreeformData` (Which is automatically implemented when types implement 
`Sync + Send + 'static + DeserializeOwned + Serialize`). These trait bounds should be easy enough for objects that
are primarily for storing data.

The data is stored in the `Sord` (Serialized OR Deserialized) type, which keeps a cached value of the type as either the
serialized string, the deserialized value, or both, and uses `OnceLocks` to only generate the se/de alternate type when
requested. 

`Freeform` and `Sord` both have a `SerdeScheme` type parameter to determine how stored values are 
serialized/deserialized, but `Freeform` uses `Json` (`serde_json`) by default. This crate also provides `Toml` and `Ron`
implementations. 


`Freeform` is implemented to look naturally when serialized with the corresponding scheme. If serialized with a different
scheme, no behavior is guaranteed, so `Freeform` also provides helper methods to serialize/deserialize itself using the 
same scheme it uses for its values

```rust
    let freeform: Freeform<Ron> = ... ;
    serde_json::to_string(freeform) // What would this look like? Use freeform.serialize() instead
```

## Future plans

In the future, there are a number of optimizations I would like to implement.

* `FreeformData: 'static` to `FreeformData<'a>`
* `Svord<S: SerdeScheme>` (Stores `S::Value` as well)
* `ron` and `toml` features to reduce unneeded dependencies