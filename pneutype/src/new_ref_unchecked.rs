use crate::Validate;

pub trait NewRefUnchecked {
    type Input: ?Sized;
    unsafe fn new_ref_unchecked(input: &Self::Input) -> &Self;
}

impl NewRefUnchecked for str {
    type Input = str;
    unsafe fn new_ref_unchecked(input: &Self::Input) -> &Self {
        debug_assert!(
            <Self as Validate>::validate(input).is_ok(),
            "programmer error: new_ref_unchecked was passed invalid data"
        );
        // See https://stackoverflow.com/questions/64977525/how-can-i-create-newtypes-for-an-unsized-type-and-its-owned-counterpart-like-s
        &*(input as *const str as *const Self)
    }
}
