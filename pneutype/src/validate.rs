use crate::PneuString;

/// Used to define the validation constraint for use in `PneuStr`, as well as the Error type for when validation fails.
pub trait Validate {
    type Data: ?Sized;
    type Error: std::fmt::Debug + std::fmt::Display;
    fn validate(data: &Self::Data) -> Result<(), Self::Error>;
}

/// Canonical implementation of Validate for str that never fails.
impl Validate for str {
    type Data = str;
    type Error = std::convert::Infallible;
    fn validate(_data: &Self::Data) -> Result<(), Self::Error> {
        // Always valid.
        Ok(())
    }
}

/// A PneuString inherits its PneuStr's impl of Validate.
impl<T: PneuString> Validate for T {
    type Data = <<T as PneuString>::Borrowed as Validate>::Data;
    type Error = <<T as PneuString>::Borrowed as Validate>::Error;
    fn validate(data: &Self::Data) -> Result<(), Self::Error> {
        <<T as PneuString>::Borrowed as Validate>::validate(data)
    }
}
