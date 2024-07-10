use pneutype_derive_tests::{Lowercase, LowercaseStr};
use std::{borrow::Cow, str::FromStr};

#[test]
fn test_pneu_string_and_pneu_str_1() {
    let t0 = Lowercase::try_from("abcd").expect("pass");
    println!("t0 (as Debug): {:?}", t0);
    // Ensure can use the chain of derefs Lowercase -> LowercaseStr -> str in order to call a str method.
    assert_eq!(t0.len(), t0.as_str().len());

    println!("t0 (as Display): {}", t0);

    Lowercase::try_from("abcdE").expect_err("pass");
    Lowercase::new("abcdE".to_string()).expect_err("pass");
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

    let r2 = LowercaseStr::new_ref("abcd").expect("pass");
    assert_eq!(r0, r2);

    let t2 = r0.to_owned();
    assert_eq!(t0, t2);

    let s0 = t0.into_string();
    assert_eq!(s0.as_str(), t2.as_str());
}

#[derive(Debug, Eq, PartialEq, pneutype::PneuString)]
#[pneu_string(borrow = "URLStr")]
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
        URL::new(invalid_url_str.to_string()).expect_err("pass");
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
        URL::try_from(url_str.to_string()).expect("pass");
        <&URLStr>::try_from(url_str).expect("pass");
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
