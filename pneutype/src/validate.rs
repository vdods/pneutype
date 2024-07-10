/// Used to define the validation constraint for use in `PneuStr`, as well as the Error type for when validation fails.
pub trait Validate {
    type Data: ?Sized;
    type Error;
    fn validate(data: &Self::Data) -> Result<(), Self::Error>;
}
