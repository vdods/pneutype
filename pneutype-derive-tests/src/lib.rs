/// A string that is_ascii_lowercase, i.e. has no non-ascii-lowercase chars.
#[derive(Debug, Eq, PartialEq, Hash, pneutype::PneuString, serde::Serialize)]
#[pneu_string(borrow = "LowercaseStr")]
pub struct Lowercase(String);

impl Lowercase {
    /// Reversing the order of the chars doesn't affect lowercasedness, so the validation constraint will be respected.
    pub fn reverse(&mut self) {
        // Unsafe because it's up to the programmer to get this right.
        *self = unsafe { Self::new_unchecked(self.chars().rev().collect()) };
    }
}

struct LowercaseVisitor;

impl<'a> serde::de::Visitor<'a> for LowercaseVisitor {
    type Value = Lowercase;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string")
    }
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Lowercase::try_from(v).map_err(serde::de::Error::custom)
    }
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Lowercase::new(v).map_err(serde::de::Error::custom)
    }
}

impl<'de> serde::Deserialize<'de> for Lowercase {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_string(LowercaseVisitor)
    }
}

/// The str-equivalent of Lowercase.  Is used to pass validated-lowercase strings by reference.
#[derive(Debug, Eq, PartialEq, Hash, pneutype::PneuStr, serde::Serialize)]
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

struct LowercaseStrVisitor;

impl<'a> serde::de::Visitor<'a> for LowercaseStrVisitor {
    type Value = &'a LowercaseStr;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a borrowed lowercase string")
    }
    fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        LowercaseStr::new_ref(v).map_err(serde::de::Error::custom)
    }
}

impl<'de: 'a, 'a> serde::Deserialize<'de> for &'a LowercaseStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(LowercaseStrVisitor)
    }
}
