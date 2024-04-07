mod freeform;
mod scheme;
mod sord;
mod typed_sord;

pub use freeform::*;
pub use scheme::*;
pub use sord::*;
pub use typed_sord::*;

/// Simple alias for Freeform<Json>
///
/// This is also the default for Freeform
#[cfg(feature = "json")]
pub type FreeformJson = Freeform<Json>;

/// Simple alias for Freeform<Toml>
#[cfg(feature = "toml")]
pub type FreeformToml = Freeform<Toml>;

/// Simple alias for Freeform<Ron>
#[cfg(feature = "ron")]
pub type FreeformRon = Freeform<Ron>;
