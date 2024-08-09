use crate::{AsStr, NewRefUnchecked, Validate};

pub trait PneuStr:
    AsRef<str>
    + AsStr
    + std::borrow::Borrow<str>
    + std::fmt::Display
    + NewRefUnchecked<Input = str>
    + Validate<Data = str, Error = Self::ValidateError>
{
    type ValidateError: std::fmt::Debug + std::fmt::Display;
    fn new_ref(s: &str) -> Result<&Self, <Self as Validate>::Error>;
}

/// Automatic implementation of PneuStr for any type that implements appropriate traits.
impl<T> PneuStr for T
where
    T: AsRef<str>
        + AsStr
        + std::borrow::Borrow<str>
        + std::fmt::Display
        + NewRefUnchecked<Input = str>
        + ?Sized
        + Validate<Data = str>,
    <T as Validate>::Error: std::fmt::Debug + std::fmt::Display,
{
    type ValidateError = <T as Validate>::Error;
    fn new_ref(s: &str) -> Result<&Self, <Self as Validate>::Error> {
        <Self as Validate>::validate(s)?;
        unsafe { Ok(Self::new_ref_unchecked(s)) }
    }
}
