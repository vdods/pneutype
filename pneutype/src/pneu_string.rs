use crate::{AsStr, PneuStr};

pub trait PneuString:
    AsRef<Self::Borrowed>
    + AsRef<str>
    + AsStr
    + std::borrow::Borrow<Self::Borrowed>
    + std::borrow::Borrow<str>
    + std::ops::Deref<Target = Self::Borrowed>
    + std::fmt::Display
    + std::str::FromStr<Err = Self::FromStrErr>
    + TryFrom<String, Error = Self::TryFromStringErr>
{
    type Borrowed: PneuStr + ?Sized;
    type FromStrErr: std::fmt::Debug + std::fmt::Display;
    type TryFromStringErr: std::fmt::Debug + std::fmt::Display;
    unsafe fn new_unchecked(s: String) -> Self;
    fn as_pneu_str(&self) -> &Self::Borrowed;
    fn into_string(self) -> String;
}

impl PneuString for String {
    type Borrowed = str;
    type FromStrErr = <Self as std::str::FromStr>::Err;
    type TryFromStringErr = <Self as TryFrom<String>>::Error;
    unsafe fn new_unchecked(s: String) -> Self {
        s
    }
    fn as_pneu_str(&self) -> &Self::Borrowed {
        self.as_str()
    }
    fn into_string(self) -> String {
        self
    }
}
