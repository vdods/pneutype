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
