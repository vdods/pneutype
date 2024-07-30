/// A string that is_ascii_lowercase, i.e. has no non-ascii-lowercase chars.
#[derive(Debug, Eq, PartialEq, Hash, pneutype::PneuString, serde::Serialize)]
#[pneu_string(borrow = "LowercaseStr", deserialize)]
pub struct Lowercase(String);

impl Lowercase {
    /// Reversing the order of the chars doesn't affect lowercasedness, so the validation constraint will be respected.
    pub fn reverse(&mut self) {
        // Unsafe because it's up to the programmer to get this right.
        *self = unsafe { Self::new_unchecked(self.chars().rev().collect()) };
    }
}

/// The str-equivalent of Lowercase.  Is used to pass validated-lowercase strings by reference.
#[derive(Debug, Eq, PartialEq, Hash, pneutype::PneuStr, serde::Serialize)]
#[pneu_str(deserialize)]
#[repr(transparent)] // `repr(transparent)` is required for PneuStr!
pub struct LowercaseStr(str);

impl LowercaseStr {
    /// You can define whatever methods you want.
    pub fn count_lowercase_chars(&self) -> usize {
        let mut n = 0;
        for c in self.chars() {
            if c.is_ascii_lowercase() {
                n += 1;
            }
        }
        n
    }
}

impl pneutype::Validate for LowercaseStr {
    type Data = str;
    type Error = &'static str;
    fn validate(data: &Self::Data) -> Result<(), Self::Error> {
        if data.chars().all(|c| c.is_ascii_lowercase()) {
            Ok(())
        } else {
            Err("must be an all-lowercase string")
        }
    }
}

// Generics with PneuString and PneuStr

/// A string that can be parsed into a value of type T: std::str::FromStr.
#[derive(Debug, Eq, PartialEq, Hash, pneutype::PneuString, serde::Serialize)]
#[pneu_string(borrow = "ValueStr", deserialize, string_field = "1")]
pub struct ValueString<T: 'static + std::str::FromStr>(std::marker::PhantomData<T>, String);

impl<T: std::str::FromStr> ValueString<T> {
    /// Can set this ValueString using the given value and its constraint will still be satisfied.
    /// This will still check the validity condition, because there's no guarantee that
    /// <T as std::fmt::Display> will produce the exact string that <T as std::str::FromStr> expects.
    pub fn set_value(&mut self, value: &T)
    where
        T: std::fmt::Display,
    {
        let s = value.to_string();
        <ValueStr<T> as pneutype::Validate>::validate(s.as_str()).expect("programmer error: <T as std::fmt::Display> is inconsistent with <T as std::str::FromStr>");
        self.1 = s;
    }
}

/// The str-equivalent of ValueString.  Is used to pass validated-ValueString strings by reference.
#[derive(Debug, Eq, PartialEq, Hash, pneutype::PneuStr, serde::Serialize)]
#[pneu_str(deserialize, str_field = "1")]
#[repr(transparent)] // `repr(transparent)` is required for PneuStr!
pub struct ValueStr<T: 'static + std::str::FromStr>(std::marker::PhantomData<T>, str);

impl<T> ValueStr<T>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    pub fn to_value(&self) -> T {
        T::from_str(self.as_str()).expect("programmer error")
    }
}

impl<T> pneutype::Validate for ValueStr<T>
where
    T: std::str::FromStr,
{
    type Data = str;
    type Error = &'static str;
    fn validate(data: &Self::Data) -> Result<(), Self::Error> {
        T::from_str(data).map_err(|_| "parse error in value")?;
        Ok(())
    }
}
