use pneutype_derive_tests::{Lowercase, LowercaseStr, ValueStr, ValueString};
use std::{borrow::Cow, str::FromStr};

#[test]
fn test_pneu_string_and_pneu_str_1() {
    let t0 = Lowercase::try_from("abcd").expect("pass");
    println!("t0 (as Debug): {:?}", t0);
    // Ensure can use the chain of derefs Lowercase -> LowercaseStr -> str in order to call a str method.
    assert_eq!(t0.len(), t0.as_str().len());

    println!("t0 (as Display): {}", t0);

    Lowercase::try_from("abcdE").expect_err("pass");
    <&LowercaseStr>::try_from("abcdE").expect_err("pass");
    LowercaseStr::new_ref("abcdE").expect_err("pass");

    use std::ops::Deref;
    let r0 = t0.deref();
    println!("r0 (as Debug): {:?}", r0);
    println!("r0 (as Display): {}", r0);

    let t1 = Lowercase::from_str("abcd").expect("pass");
    assert_eq!(t0, t1);

    let r1 = <&LowercaseStr>::try_from("abcd").expect("pass");
    assert_eq!(r0, r1);
    assert_eq!(t1.as_pneu_str(), r1);

    let r2 = LowercaseStr::new_ref("abcd").expect("pass");
    assert_eq!(r0, r2);

    let t2 = r0.to_owned();
    assert_eq!(t0, t2);

    let s0 = t0.into_string();
    assert_eq!(s0.as_str(), t2.as_str());
}

fn do_stuff_with_lowercase_str(_: &LowercaseStr) {
    // Actually do nothing, ha ha ha!
}

#[test]
fn test_pneu_string_borrow() {
    let t0 = Lowercase::try_from("abcd").expect("pass");
    do_stuff_with_lowercase_str(&t0);
    let r0 = &t0;
    do_stuff_with_lowercase_str(r0);
}

#[derive(Debug, Eq, PartialEq, pneutype::PneuString)]
#[pneu_string(borrow = "URLStr", as_pneu_str = "as_url_str")]
struct URL(String);

#[derive(Debug, Eq, PartialEq, pneutype::PneuStr)]
#[repr(transparent)] // `repr(transparent)` is required for PneuStr!
struct URLStr(str);

impl pneutype::Validate for URLStr {
    type Data = str;
    type Error = Cow<'static, str>;
    fn validate(data: &Self::Data) -> Result<(), Self::Error> {
        url::Url::parse(data).map_err(|e| std::borrow::Cow::Owned(e.to_string()))?;
        Ok(())
    }
}

impl URLStr {
    pub fn scheme(&self) -> &str {
        self.0.split_once("://").expect("programmer error").0
    }
    pub fn host(&self) -> &str {
        let (_scheme, after_scheme) = self.0.split_once("://").expect("programmer error");
        let n = after_scheme.find(':').unwrap_or(after_scheme.len());
        let n = n.min(after_scheme.find('/').unwrap_or(after_scheme.len()));
        let n = n.min(after_scheme.find('?').unwrap_or(after_scheme.len()));
        let n = n.min(after_scheme.find('#').unwrap_or(after_scheme.len()));
        &after_scheme[..n]
    }
    pub fn port(&self) -> Option<u16> {
        let (_scheme, after_scheme) = self.0.split_once("://").expect("programmer error");
        if let Some((_before_colon, after_colon)) = after_scheme.split_once(':') {
            let n = after_colon.find('/').unwrap_or(after_colon.len());
            let n = n.min(after_colon.find('?').unwrap_or(after_colon.len()));
            let n = n.min(after_colon.find('#').unwrap_or(after_colon.len()));
            Some(u16::from_str(&after_colon[..n]).expect("programmer error"))
        } else {
            None
        }
    }
    pub fn path(&self) -> Option<&str> {
        let (_scheme, after_scheme) = self.0.split_once("://").expect("programmer error");
        if let Some(path_start) = after_scheme.find('/') {
            let path_end = self
                .0
                .find('?')
                .unwrap_or(after_scheme.len())
                .min(after_scheme.find('#').unwrap_or(after_scheme.len()));
            Some(&after_scheme[path_start..path_end])
        } else {
            None
        }
    }
    pub fn query(&self) -> Option<&str> {
        if let Some((rest, _fragment)) = self.0.rsplit_once('#') {
            rest.rsplit_once('?').map(|(_rest, query)| query)
        } else {
            self.0.rsplit_once('?').map(|(_rest, query)| query)
        }
    }
    pub fn fragment(&self) -> Option<&str> {
        self.0.rsplit_once('#').map(|(_rest, fragment)| fragment)
    }
}

#[test]
fn test_pneu_string_and_pneu_str_2() {
    const INVALID_URL_STR_V: &[&str] =
        &["xyz", "http::blah/123", "http::/blah/123", "blah/123?a=b"];

    for &invalid_url_str in INVALID_URL_STR_V.iter() {
        println!("invalid_url_str: {:?}", invalid_url_str);
        URL::from_str(invalid_url_str).expect_err("pass");
        URL::try_from(invalid_url_str).expect_err("pass");
        <&URLStr>::try_from(invalid_url_str).expect_err("pass");
        URLStr::new_ref(invalid_url_str).expect_err("pass");
    }

    const URL_STR_V: &[&str] = &[
        "https://no.golf:12345/at/all?abc=pqr#tag1",
        "https://no.golf:12345/at/all?abc=pqr",
        "https://no.golf:12345/at/all#tag1",
        "https://no.golf:12345/at/all",
        "https://no.golf:12345?abc=pqr#tag1",
        "https://no.golf:12345?abc=pqr",
        "https://no.golf:12345#tag1",
        "https://no.golf:12345",
        "https://no.golf/at/all?abc=pqr#tag1",
        "https://no.golf/at/all?abc=pqr",
        "https://no.golf/at/all#tag1",
        "https://no.golf/at/all",
        "https://no.golf?abc=pqr#tag1",
        "https://no.golf?abc=pqr",
        "https://no.golf#tag1",
        "https://no.golf",
        "file:///a/b/c",
    ];

    for &url_str in URL_STR_V.iter() {
        URL::from_str(url_str).expect("pass");
        URL::try_from(url_str).expect("pass");
        let u0 = URL::try_from(url_str.to_string()).expect("pass");
        let u1 = <&URLStr>::try_from(url_str).expect("pass");
        assert_eq!(u0.as_url_str(), u1);
        URLStr::new_ref(url_str).expect("pass");

        let t0 = URL::try_from(url_str).expect("pass");
        println!("t0: {:?}", t0);

        use std::ops::Deref;
        let r0 = t0.deref();
        println!("r0: {:?}", r0);

        assert_eq!(t0.scheme(), r0.scheme());
        assert_eq!(t0.host(), r0.host());
        assert_eq!(t0.port(), r0.port());
        assert_eq!(t0.path(), r0.path());
        assert_eq!(t0.query(), r0.query());
        assert_eq!(t0.fragment(), r0.fragment());

        let t1 = URL::from_str(url_str).expect("pass");
        assert_eq!(t0, t1);

        let r1 = <&URLStr>::try_from(url_str).expect("pass");
        assert_eq!(r0, r1);

        let r2 = URLStr::new_ref(url_str).expect("pass");
        assert_eq!(r0, r2);

        let t2 = r0.to_owned();
        assert_eq!(t0, t2);

        let s0 = t0.into_string();
        assert_eq!(s0.as_str(), t2.as_str());
    }
}

fn do_stuff(lowercase_str: &LowercaseStr) {
    println!("lowercase_str: {:?}", lowercase_str);
}

#[test]
fn test_pneu_string_and_pneu_str_3() {
    let lowercase = Lowercase::from_str("xyz").expect("pass");
    do_stuff(&lowercase);
}

// TODO: Figure out how to cause Cow<'a, LowercaseStr> to be borrowed upon deserialize.
// See:
// -    https://github.com/serde-rs/serde/issues/1852
// -    https://play.rust-lang.org/?version=nightly&mode=debug&edition=2018&gist=418dd6b98dfa62d43c4cc7fa8b7ea0d6
#[derive(Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
struct Thingy<'a> {
    lowercase: Lowercase,
    #[serde(borrow)]
    lowercase_str: &'a LowercaseStr,
    #[serde(borrow)]
    lowercase_cow_1: Cow<'a, LowercaseStr>,
    #[serde(borrow)]
    lowercase_cow_2: Cow<'a, LowercaseStr>,
    #[serde(borrow)]
    str_cow_1: Cow<'a, str>,
    #[serde(borrow)]
    str_cow_2: Cow<'a, str>,
}

#[test]
fn test_pneu_string_and_pneu_str_4() {
    let thingy = Thingy {
        lowercase: Lowercase::from_str("xyz").expect("pass"),
        lowercase_str: LowercaseStr::new_ref("pqr").expect("pass"),
        lowercase_cow_1: Cow::Owned(Lowercase::from_str("abc").expect("pass")),
        lowercase_cow_2: Cow::Borrowed(LowercaseStr::new_ref("uvw").expect("pass")),
        str_cow_1: Cow::Owned("LMN".to_string()),
        str_cow_2: Cow::Borrowed("EFG"),
    };
    println!("thingy: {:?}", thingy);
    println!(
        "thingy.lowercase_cow_1 is borrowed: {}",
        matches!(thingy.lowercase_cow_1, Cow::Borrowed(_))
    );
    println!(
        "thingy.lowercase_cow_2 is borrowed: {}",
        matches!(thingy.lowercase_cow_2, Cow::Borrowed(_))
    );
    println!(
        "thingy.str_cow_1 is borrowed: {}",
        matches!(thingy.str_cow_1, Cow::Borrowed(_))
    );
    println!(
        "thingy.str_cow_2 is borrowed: {}",
        matches!(thingy.str_cow_2, Cow::Borrowed(_))
    );
    let thingy_json = serde_json::to_string(&thingy).expect("pass");
    println!("thingy_json: {}", thingy_json);
    let thingy_deserialized: Thingy = serde_json::from_str(&thingy_json).expect("pass");
    println!(
        "thingy_deserialized.lowercase_cow_1 is borrowed: {}",
        matches!(thingy_deserialized.lowercase_cow_1, Cow::Borrowed(_))
    );
    println!(
        "thingy_deserialized.lowercase_cow_2 is borrowed: {}",
        matches!(thingy_deserialized.lowercase_cow_2, Cow::Borrowed(_))
    );
    println!(
        "thingy_deserialized.str_cow_1 is borrowed: {}",
        matches!(thingy_deserialized.str_cow_1, Cow::Borrowed(_))
    );
    println!(
        "thingy_deserialized.str_cow_2 is borrowed: {}",
        matches!(thingy_deserialized.str_cow_2, Cow::Borrowed(_))
    );
    assert_eq!(thingy, thingy_deserialized);
}

// Ensure it's possible to have a free-standing PneuStr.
#[derive(Debug, Eq, PartialEq, pneutype::PneuStr)]
#[repr(transparent)]
struct FreeStandingStr(str);

impl pneutype::Validate for FreeStandingStr {
    type Data = str;
    type Error = &'static str;
    fn validate(data: &Self::Data) -> Result<(), Self::Error> {
        if data.is_empty() {
            Err("empty string is not allowed".into())
        } else {
            Ok(())
        }
    }
}

#[test]
fn test_pneu_string_and_pneu_str_5() {
    let x = FreeStandingStr::new_ref("stuff").expect("pass");
    assert_eq!(x.as_str(), "stuff");
}

#[derive(Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
struct StronglyTyped<'a> {
    lowercase: Lowercase,
    #[serde(borrow)]
    lowercase_str: &'a LowercaseStr,
}

#[derive(Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
struct WeaklyTyped<'a> {
    lowercase: String,
    #[serde(borrow)]
    lowercase_str: &'a str,
}

#[test]
fn test_pneu_string_and_pneu_str_serde() {
    // Verify that the validation works as expected when deserializing.

    {
        let w = WeaklyTyped {
            lowercase: String::from_str("blahblah").expect("pass"),
            lowercase_str: "heyhey",
        };
        let json = serde_json::to_string(&w).expect("pass");
        let s: StronglyTyped = serde_json::from_str(&json).expect("pass");
        assert_eq!(s.lowercase.as_str(), w.lowercase.as_str());
        assert_eq!(s.lowercase_str.as_str(), w.lowercase_str);
    }
    {
        let w = WeaklyTyped {
            lowercase: String::from_str("NOT LOWERCASE").expect("pass"),
            lowercase_str: "heyhey",
        };
        let json = serde_json::to_string(&w).expect("pass");
        let err = serde_json::from_str::<StronglyTyped>(&json).expect_err("pass");
        println!("serde_json::from_str err (expected): {}", err);
    }
    {
        let w = WeaklyTyped {
            lowercase: String::from_str("blahblah").expect("pass"),
            lowercase_str: "I AM YELLING",
        };
        let json = serde_json::to_string(&w).expect("pass");
        let err = serde_json::from_str::<StronglyTyped>(&json).expect_err("pass");
        println!("serde_json::from_str err (expected): {}", err);
    }
    {
        let w = WeaklyTyped {
            lowercase: String::from_str("NOT LOWERCASE").expect("pass"),
            lowercase_str: "I AM YELLING",
        };
        let json = serde_json::to_string(&w).expect("pass");
        let err = serde_json::from_str::<StronglyTyped>(&json).expect_err("pass");
        println!("serde_json::from_str err (expected): {}", err);
    }
}

#[test]
fn test_pneu_str_with_generics() {
    type I32Str = ValueStr<i32>;

    I32Str::new_ref("").expect_err("pass");
    I32Str::new_ref("abc").expect_err("pass");
    I32Str::new_ref("12.34").expect_err("pass");

    let i = I32Str::new_ref("123").expect("pass");
    assert_eq!(i.as_str(), "123");
    assert_eq!(i.to_value(), 123i32);

    type BoolStr = ValueStr<bool>;

    BoolStr::new_ref("").expect_err("pass");
    BoolStr::new_ref("blah").expect_err("pass");
    BoolStr::new_ref("1").expect_err("pass");

    let b = BoolStr::new_ref("true").expect("pass");
    assert_eq!(b.as_str(), "true");
    assert_eq!(b.to_value(), true);
}

#[test]
fn test_pneu_string_with_generics() {
    type I32String = ValueString<i32>;
    type I32Str = ValueStr<i32>;

    I32String::try_from("").expect_err("pass");
    I32String::try_from("abc").expect_err("pass");
    I32String::try_from("12.34").expect_err("pass");

    let mut i = I32String::try_from("123").expect("pass");
    assert_eq!(i.as_pneu_str(), I32Str::new_ref("123").expect("pass"));
    assert_eq!(i.as_str(), "123");
    assert_eq!(i.to_value(), 123i32);
    i.set_value(&456i32);
    assert_eq!(i.as_pneu_str(), I32Str::new_ref("456").expect("pass"));
    assert_eq!(i.as_str(), "456");
    assert_eq!(i.to_value(), 456i32);

    type BoolString = ValueString<bool>;
    type BoolStr = ValueStr<bool>;

    BoolString::try_from("").expect_err("pass");
    BoolString::try_from("blah").expect_err("pass");
    BoolString::try_from("1").expect_err("pass");

    let mut i = BoolString::try_from("true").expect("pass");
    assert_eq!(i.as_pneu_str(), BoolStr::new_ref("true").expect("pass"));
    assert_eq!(i.as_str(), "true");
    assert_eq!(i.to_value(), true);
    i.set_value(&false);
    assert_eq!(i.as_pneu_str(), BoolStr::new_ref("false").expect("pass"));
    assert_eq!(i.as_str(), "false");
    assert_eq!(i.to_value(), false);
}

#[test]
fn test_pneu_string_with_generics_serde() {
    type I32String = ValueString<i32>;
    type I32Str = ValueStr<i32>;

    {
        let i = I32String::try_from("123").expect("pass");
        println!("i: {:?}", i);
        assert_eq!(i.as_pneu_str(), I32Str::new_ref("123").expect("pass"));
        let i_json = serde_json::to_string(&i).expect("pass");
        println!("i_json: {}", i_json);
        assert_eq!(i_json, "\"123\"");
        let i_deserialized: I32String = serde_json::from_str(&i_json).expect("pass");
        println!("i_deserialized: {:?}", i_deserialized);
        assert_eq!(i_deserialized, i);
    }

    {
        let i = I32Str::new_ref("123").expect("pass");
        println!("i: {:?}", i);
        let i_json = serde_json::to_string(&i).expect("pass");
        println!("i_json: {}", i_json);
        assert_eq!(i_json, "\"123\"");
        let i_deserialized: &I32Str = serde_json::from_str(&i_json).expect("pass");
        println!("i_deserialized: {:?}", i_deserialized);
        assert_eq!(i_deserialized, i);
    }
}

#[test]
fn test_pneu_str_trait() {
    let x = <LowercaseStr as pneutype::PneuStr>::new_ref("abcxyz").expect("pass");
    assert_eq!(<LowercaseStr as pneutype::AsStr>::as_str(x), "abcxyz");

    let y = unsafe { <LowercaseStr as pneutype::NewRefUnchecked>::new_ref_unchecked("abcpqr") };
    assert_eq!(<LowercaseStr as pneutype::AsStr>::as_str(y), "abcpqr");
}

fn test_pneu_str_trait_case<T>(valid_str: &str, invalid_str: &str)
where
    T: std::fmt::Debug + pneutype::PneuStr + ?Sized,
{
    let x = T::new_ref(valid_str).expect("pass");
    assert_eq!(x.as_str(), valid_str);

    T::new_ref(invalid_str).expect_err("pass");

    let y = unsafe { T::new_ref_unchecked(valid_str) };
    assert_eq!(y.as_str(), valid_str);
    assert_eq!(<T as AsRef<str>>::as_ref(&y), valid_str);
}

fn test_pneu_string_trait_case<T>(valid_str: &str, invalid_str: &str)
where
    T: std::fmt::Debug + pneutype::PneuString,
    T::Borrowed: std::fmt::Debug,
{
    let x = T::try_from(valid_str.to_string()).expect("pass");
    assert_eq!(x.as_str(), valid_str);
    let x_string = x.into_string();
    assert_eq!(x_string.as_str(), valid_str);

    T::try_from(invalid_str.to_string()).expect_err("pass");
    T::from_str(invalid_str).expect_err("pass");

    let y = unsafe { T::new_unchecked(valid_str.to_string()) };
    assert_eq!(y.as_str(), valid_str);
    let y_ref = <T as AsRef<T::Borrowed>>::as_ref(&y);
    {
        use pneutype::AsStr;
        assert_eq!(y_ref.as_str(), valid_str);
    }
    assert_eq!(<T as AsRef<str>>::as_ref(&y), valid_str);

    let z = y.as_pneu_str();
    {
        use pneutype::AsStr;
        assert_eq!(z.as_str(), valid_str);
    }

    test_pneu_str_trait_case::<T::Borrowed>(valid_str, invalid_str);
}

#[test]
fn test_pneu_string_trait_lowercase() {
    test_pneu_string_trait_case::<Lowercase>("abcxyz", "abcPqr");
}

#[test]
fn test_pneu_string_trait_value_string_i32() {
    test_pneu_string_trait_case::<ValueString<i32>>("123", "345.6");
}

#[test]
fn test_pneu_string_trait_value_string_f32() {
    test_pneu_string_trait_case::<ValueString<f32>>("12.25", "abc");
}

#[test]
fn test_pneu_str_trait_value_string_f32() {
    test_pneu_str_trait_case::<FreeStandingStr>("blah", "");
}
