//! The `pneutype` crate provides proc-macros for easily creating constrained newtypes analogous to String and &str,
//! which will be referred to generally as [PneuString] and [PneuStr] respectively, or generically as a pneutype.
//!
//! Constraints are defined using the [Validate] trait.  The proc-macros [PneuString] and [PneuStr] are used
//! to implement the rest of the traits needed for the [PneuString]-[PneuStr] behavior.  Once defined, the [PneuString]
//! and [PneuStr] can be used in place of [String] and [str], respectively, with the guarantee that the validation
//! constraint is still enforced.  This also avoids the situation in which an "owned" newtype must be allocated and
//! passed in by reference (e.g. `&url::Url`) in a function that requires valid data but doesn't take ownership over
//! that data.
//!
//! Appropriate, natural impls of of [Borrow](std::borrow::Borrow), [Deref](std::ops::Deref),
//! [Display](std::fmt::Display), [From], [FromStr](std::str::FromStr), [ToOwned], and [TryFrom] are provided for the
//! newtypes, so that they can still transparently be used as if they were [String] and [str].
//!
//! Minimal example:
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
//!
//! [serde::Serialize](https://docs.rs/serde/latest/serde/trait.Serialize.html) can be implemented directly on the
//! pneutype via the standard derive.  A validating implementation of
//! [serde::Deserialize](https://docs.rs/serde/latest/serde/trait.Deserialize.html) can be added by specifying the
//! `deserialize` attribute under the appropriate proc macro attribute (`pneu_string` and `pneu_str` respectively).  It
//! is incorrect to derive [serde::Deserialize](https://docs.rs/serde/latest/serde/trait.Deserialize.html) directly on
//! the pneutype, as the standard derived implementation does no pneutype-specific validation.
//!
//! Example involving `serde`-based serialization and deserialization:
//! ```
//! #[derive(Debug, PartialEq, pneutype::PneuString, serde::Serialize)]
//! #[pneu_string(borrow = "SplungeStr", deserialize)]
//! struct Splunge(String);
//!
//! #[derive(Debug, PartialEq, pneutype::PneuStr, serde::Serialize)]
//! #[pneu_str(deserialize)]
//! #[repr(transparent)] // `repr(transparent)` is required for PneuStr!
//! struct SplungeStr(str);
//!
//! impl pneutype::Validate for SplungeStr {
//!     type Data = str;
//!     type Error = &'static str;
//!     fn validate(data: &Self::Data) -> Result<(), Self::Error> {
//!         // Let's define a Splunge to be a non-empty string.
//!         if data.is_empty() {
//!             Err("a Splunge must a non-empty")
//!         } else {
//!             Ok(())
//!         }
//!     }
//! }
//!
//! #[derive(Debug, serde::Deserialize, PartialEq, serde::Serialize)]
//! struct Document<'a> {
//!     splunge: Splunge,
//!     #[serde(borrow)]
//!     splunge_str: &'a SplungeStr,
//! }
//!
//! fn demonstrate_serde() {
//!     let document = Document {
//!         splunge: Splunge::try_from("stuff".to_string()).expect("pass"),
//!         splunge_str: SplungeStr::new_ref("and things").expect("pass"),
//!     };
//!     let json = serde_json::to_string(&document).expect("pass");
//!     println!("document JSON: {}", json);
//!     // Note that the deserialization will invoke the validation checks because of the
//!     // pneutype-based impl of serde::Deserialize.
//!     let document_deserialized = serde_json::from_str::<Document>(&json).expect("pass");
//!     assert_eq!(document_deserialized, document);
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
///
/// Attributes for `pneu_string`:
/// -   borrow = "..." -- this should specify the type name of the corresponding [PneuStr].
///     In the above example that would be `ThingyStr`.
/// -   deserialize -- if present, then the proc-macro will generate an implementation of
///     [serde::Deserialize](https://docs.rs/serde/latest/serde/trait.Deserialize.html)
///     performs the expected validation (in particular, returning error if the validation constraints are not met).
///
/// Note that [serde::Serialize](https://docs.rs/serde/latest/serde/trait.Serialize.html) can be implemented
/// directly on the [PneuString] via the standard derive.
pub use pneutype_derive::PneuString;

/// This will define a str-based newtype, which will be referred to generally as a "PneuStr".  A [PneuStr] can be
/// stand-alone; there is no requirement for a corresponding [PneuString].  Note that `repr(transparent)` is
/// required for [PneuStr]!
///
/// The
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
///
/// Attributes for `pneu_str`:
/// -   deserialize -- if present, then the proc-macro will generate an implementation of
///     [serde::Deserialize](https://docs.rs/serde/latest/serde/trait.Deserialize.html)
///     performs the expected validation (in particular, returning error if the validation constraints are not met).
///
/// Note that [serde::Serialize](https://docs.rs/serde/latest/serde/trait.Serialize.html) can be implemented
/// directly on the [PneuStr] via the standard derive.
pub use pneutype_derive::PneuStr;

pub use crate::validate::Validate;
