//! The `pneutype` crate provides proc-macros for easily creating constrained newtypes analogous to String and &str,
//! which will be referred to generally as [PneuString] and [PneuStr] respectively.
//!
//! Constraints are defined using the [Validate] trait.  The proc-macros [PneuString] and [PneuStr] are used
//! to implement the rest of the traits needed for the [PneuString]-[PneuStr] behavior.  Once defined, the [PneuString]
//! and [PneuStr] can be used in place of [String] and [str], respectively, with the guarantee that the validation
//! constraint is still enforced.  This also avoids the situation in which an "owned" newtype must be allocated and
//! passed in by reference (e.g. `&url::Url`) in a function that requires valid data but doesn't take ownership over
//! that data.
//!
//! Appropriate, natural impls of of [Borrow](std::borrow::Borrow), [Deref](std::ops::Deref), [Display](std::fmt::Display),
//! [From](std::convert::From), [FromStr](std::str::FromStr), [ToOwned](std::borrow::ToOwned), and
//! [TryFrom](std::convert::TryFrom) are provided for the newtypes, so that they can still transparently be used as
//! if they were [String] and [str].
//!
//! ```
//! #[derive(pneutype::PneuString)]
//! #[pneu_string(borrow = "ThingyStr")]
//! struct Thingy(String);
//!
//! impl Thingy {
//!     // Add whatever Thingy-specific methods are appropriate, including mutating methods, so long as they
//!     // guarantee that the validation constraint is still enforced.
//! }
//!
//! #[derive(pneutype::PneuStr)]
//! #[repr(transparent)] // `repr(transparent)` is required for PneuStr!
//! struct ThingyStr(str);
//!
//! impl pneutype::Validate for ThingyStr {
//!     type Data = str;
//!     type Error = &'static str;
//!     fn validate(data: &Self::Data) -> Result<(), Self::Error> {
//!         // Enforce whatever constraints are needed by whatever a "Thingy" is.
//!         Ok(())
//!     }
//! }
//!
//! impl ThingyStr {
//!     // Add whatever ThingyStr-specific methods are appropriate, which basically means read-only methods.
//!     // This is analogous to how all read-only string methods are provided in `str`, and `String` implements
//!     // `Deref` to `str` to allow access to those methods.
//! }
//!
//! /// Use a Thingy by reference knowing that the validation constraint is still guaranteed.  Using a &str
//! /// directly which would "forget" the validation constraint.
//! fn use_thingy_str(t: &ThingyStr) {
//!     // Do something with a ThingyStr
//!
//!     // Via Deref trait, t can act as a &str as well.
//!     println!("t.len(): {}", t.len());
//!     // x will be a Thingy.
//!     let x = t.to_owned();
//!     // etc
//! }
//!
//! /// Use a Thingy as one would use a String, but with its validation constraint guaranteed.
//! fn consume_thingy(t: Thingy) {
//!     // Do something with a Thingy
//!
//!     // Use t by reference, with validation constraints still guaranteed.
//!     use_thingy_str(&t);
//!     // etc
//! }
//! ```

mod validate;

/// This will implement traits appropriate for a String-based newtype, which will be referred to generally as a "PneuString".
/// A [PneuString] always has a corresponding [PneuStr].  Trait implementation details for [PneuString] should be given via
/// the `pneu_string` attribute, e.g.
/// ```
/// #[derive(pneutype::PneuString)]
/// #[pneu_string(borrow = "ThingyStr")]
/// pub struct Thingy(String);
///
/// #[derive(pneutype::PneuStr)]
/// #[repr(transparent)] // `repr(transparent)` is required for PneuStr!
/// pub struct ThingyStr(str);
///
/// impl pneutype::Validate for ThingyStr {
///     type Data = str;
///     type Error = &'static str;
///     fn validate(data: &Self::Data) -> Result<(), Self::Error> {
///         // Enforce whatever constraints define a valid "thingy",
///         // say a string with an even number of chars.
///         if data.chars().count() % 2 == 0 {
///             Ok(())
///         } else {
///             Err("ThingyStr must have an even number of chars")
///         }
///     }
/// }
/// ```
pub use pneutype_derive::PneuString;

/// This will define a str-based newtype, which will be referred to generally as a "PneuStr".  A [PneuStr] can be
/// stand-alone; there is no requirement for a corresponding [PneuString].  Note that `repr(transparent)` is
/// required for [PneuStr]!
/// ```
/// #[derive(pneutype::PneuStr)]
/// #[repr(transparent)] // `repr(transparent)` is required for PneuStr!
/// pub struct ThingyStr(str);
///
/// impl pneutype::Validate for ThingyStr {
///     type Data = str;
///     type Error = &'static str;
///     fn validate(data: &Self::Data) -> Result<(), Self::Error> {
///         // Enforce whatever constraints define a valid "thingy",
///         // say a string with an even number of chars.
///         if data.chars().count() % 2 == 0 {
///             Ok(())
///         } else {
///             Err("ThingyStr must have an even number of chars")
///         }
///     }
/// }
/// ```
pub use pneutype_derive::PneuStr;

pub use crate::validate::Validate;
