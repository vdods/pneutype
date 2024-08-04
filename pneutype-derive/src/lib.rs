use darling::FromDeriveInput;
use quote::quote;

//
// proc_macro for creating a String-based newtype
//

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(pneu_string))]
struct PneuStringArguments {
    /// Specify the PneuStr analog to this PneuString.  This will define the target of std::borrow::Borrow and std::ops::Deref.
    borrow: String,
    /// Specify true to derive an implementation of serde::Deserialize.  The `serde` crate must be imported into
    /// the crate in which this PneuString is defined in order for this to work.  Using this attribute is optional,
    /// and a manual implementation of serde::Deserialize is of course possible.
    deserialize: bool,
    /// Specify true to derive an implementation of serde::Serialize.  The `serde` crate must be imported into
    /// the crate in which this PneuString is defined in order for this to work.  Using this attribute is optional,
    /// and a manual implementation of serde::Serialize is of course possible.  However, in the case of a PneuString
    /// with generics, this attribute must be used instead of derive(serde::Serialize) because of the presence of
    /// std::marker::PhantomData.
    serialize: bool,
    /// Optionally specify the name for a function that will return &self as a reference to the associated PneuStr.
    /// If not specified, then the name will be "as_pneu_str".
    as_pneu_str: Option<String>,
    /// Optionally specify the `String`-valued field.  If not specified, then it will be "0" (i.e. for the ordinary
    /// case of `#[derive(pneutype::PneuString)] pub struct ThingString(String);`).  This attribute
    /// would be used in the case of a PneuString having generics, e.g.
    /// `#[derive(pneutype::PneuString)] #[pneu_string(string_field = "s")] pub struct ThingString<T> { t: std::marker::PhantomData<T>, s: String }`
    string_field: Option<String>,
}

#[proc_macro_derive(PneuString, attributes(pneu_string))]
pub fn derive_pneu_string(token_stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(token_stream);
    let pneu_string_arguments =
        PneuStringArguments::from_derive_input(&input).expect("Wrong arguments");
    let pneu_string_name = input.ident;
    let (pneu_string_impl_generics, pneu_string_type_generics, pneu_string_where_clause) =
        input.generics.split_for_impl();

    use quote::ToTokens;
    let pneu_string_has_generics = !pneu_string_type_generics.to_token_stream().is_empty();
    // This assumes that the String-valued parameter for construction is named `s`.
    let pneu_string_construction = if pneu_string_has_generics {
        quote! {
            #pneu_string_name(std::marker::PhantomData, s)
        }
    } else {
        quote! {
            #pneu_string_name(s)
        }
    };
    let self_construction = if pneu_string_has_generics {
        quote! {
            Self(std::marker::PhantomData, s)
        }
    } else {
        quote! {
            Self(s)
        }
    };

    let pneu_str_name: syn::Ident = syn::parse_str(&pneu_string_arguments.borrow).unwrap();
    let string_field: syn::Expr =
        syn::parse_str(pneu_string_arguments.string_field.as_deref().unwrap_or("0")).unwrap();

    let serde_deserialize_maybe = if pneu_string_arguments.deserialize {
        // Create new lifetime parameters 'de and 'a
        let lifetime_de = syn::Lifetime::new("'de", proc_macro2::Span::call_site());
        let lifetime_a = syn::Lifetime::new("'a", proc_macro2::Span::call_site());

        let serde_deserialize_generics = {
            // Define the lifetimes with the correct relationship ('de: 'a)
            let lifetime_de_def = syn::LifetimeDef::new(lifetime_de.clone());

            // Create a new Generics object with the new lifetimes added
            let mut new_generics = input.generics.clone();
            new_generics
                .params
                .insert(0, syn::GenericParam::Lifetime(lifetime_de_def));
            new_generics
        };
        let (
            serde_deserialize_impl_generics,
            _serde_deserialize_type_generics,
            _serde_deserialize_where_clause,
        ) = serde_deserialize_generics.split_for_impl();

        use quote::ToTokens;
        let (serde_deserialize_visitor, serde_deserialize_visitor_construction) =
            if pneu_string_type_generics.to_token_stream().is_empty() {
                (quote! { struct Visitor }, quote! { Visitor })
            } else {
                (
                    quote! {
                        struct Visitor #pneu_string_impl_generics(std::marker::PhantomData #pneu_string_type_generics) #pneu_string_where_clause
                    },
                    quote! { Visitor::#pneu_string_type_generics(std::marker::PhantomData::default()) },
                )
            };

        let serde_deserialize_visitor_generics = {
            // Define the lifetimes with the correct relationship ('de: 'a)
            let lifetime_a_def = syn::LifetimeDef::new(lifetime_a.clone());

            // Create a new Generics object with the new lifetimes added
            let mut new_generics = input.generics.clone();
            new_generics
                .params
                .insert(0, syn::GenericParam::Lifetime(lifetime_a_def));
            new_generics
        };
        let (
            serde_deserialize_visitor_impl_generics,
            _serde_deserialize_visitor_type_generics,
            _serde_deserialize_visitor_where_clause,
        ) = serde_deserialize_visitor_generics.split_for_impl();

        quote! {
            impl #serde_deserialize_impl_generics serde::Deserialize<#lifetime_de> for #pneu_string_name #pneu_string_type_generics #pneu_string_where_clause {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<#lifetime_de>,
                {
                    // struct Visitor;
                    #serde_deserialize_visitor;

                    impl #serde_deserialize_visitor_impl_generics serde::de::Visitor<#lifetime_a> for Visitor #pneu_string_type_generics {
                        type Value = #pneu_string_name #pneu_string_type_generics;

                        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                            formatter.write_str("a string")
                        }
                        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                        where
                            E: serde::de::Error,
                        {
                            #pneu_string_name::try_from(v).map_err(serde::de::Error::custom)
                        }
                        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
                        where
                            E: serde::de::Error,
                        {
                            #pneu_string_name::try_from(v).map_err(serde::de::Error::custom)
                        }
                    }

                    deserializer.deserialize_string(#serde_deserialize_visitor_construction)
                }
            }
        }
    } else {
        quote! {}
    };

    let serde_serialize_maybe = if pneu_string_arguments.serialize {
        quote! {
            impl #pneu_string_impl_generics serde::Serialize for #pneu_string_name #pneu_string_type_generics #pneu_string_where_clause {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    serializer.serialize_str(self.#string_field.as_str())
                }
            }
        }
    } else {
        quote! {}
    };

    let as_pneu_str: syn::Ident = if let Some(as_pneu_str) = pneu_string_arguments.as_pneu_str {
        syn::parse_str(&as_pneu_str).unwrap()
    } else {
        syn::parse_str("as_pneu_str").unwrap()
    };

    let output = quote! {
        impl #pneu_string_impl_generics #pneu_string_name #pneu_string_type_generics #pneu_string_where_clause {
            /// Unsafe: Construct this PneuString where the input is already guaranteed (by the caller) to be valid.
            /// However, a debug_assert! will be used to check the validity condition.  For a const version of this,
            /// see new_unchecked_const.
            pub unsafe fn new_unchecked(s: String) -> Self {
                debug_assert!(<#pneu_str_name #pneu_string_type_generics as pneutype::Validate>::validate(s.as_str()).is_ok(), "programmer error: new_unchecked was passed invalid data");
                #self_construction
            }
            /// Unsafe: Construct this PneuString where the input is already guaranteed (by the caller) to be valid.
            /// Because this is a const function, the validity condition can't be checked in a debug_assert! as it
            /// is in new_ref_unchecked.
            pub const unsafe fn new_unchecked_const(s: String) -> Self {
                #self_construction
            }
            /// Return self as a reference to the associated PneuStr, i.e. a strongly-typed version of as_str.
            pub fn #as_pneu_str(&self) -> &#pneu_str_name #pneu_string_type_generics {
                use std::ops::Deref;
                self.deref()
            }
            /// Return a &str to the underlying String.
            pub fn as_str(&self) -> &str {
                self.#string_field.as_str()
            }
            /// Dissolve this instance and take the underlying String.
            pub fn into_string(self) -> String {
                self.#string_field
            }
        }

        impl #pneu_string_impl_generics std::convert::AsRef<#pneu_str_name #pneu_string_type_generics> for #pneu_string_name #pneu_string_type_generics #pneu_string_where_clause {
            fn as_ref(&self) -> &#pneu_str_name #pneu_string_type_generics {
                use std::ops::Deref;
                self.deref()
            }
        }

        impl #pneu_string_impl_generics std::convert::AsRef<str> for #pneu_string_name #pneu_string_type_generics #pneu_string_where_clause {
            fn as_ref(&self) -> &str {
                Self::as_str(self)
            }
        }

        impl #pneu_string_impl_generics std::borrow::Borrow<#pneu_str_name #pneu_string_type_generics> for #pneu_string_name #pneu_string_type_generics #pneu_string_where_clause {
            fn borrow(&self) -> &#pneu_str_name #pneu_string_type_generics {
                use std::ops::Deref;
                self.deref()
            }
        }

        impl #pneu_string_impl_generics std::borrow::Borrow<str> for #pneu_string_name #pneu_string_type_generics #pneu_string_where_clause {
            fn borrow(&self) -> &str {
                Self::as_str(self)
            }
        }

        impl #pneu_string_impl_generics std::ops::Deref for #pneu_string_name #pneu_string_type_generics #pneu_string_where_clause {
            type Target = #pneu_str_name #pneu_string_type_generics;
            fn deref(&self) -> &Self::Target {
                unsafe { #pneu_str_name::new_ref_unchecked(self.#string_field.as_str()) }
            }
        }

        #serde_deserialize_maybe

        impl #pneu_string_impl_generics std::fmt::Display for #pneu_string_name #pneu_string_type_generics #pneu_string_where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                Self::as_str(self).fmt(f)
            }
        }

        impl #pneu_string_impl_generics From<&#pneu_str_name #pneu_string_type_generics> for #pneu_string_name #pneu_string_type_generics #pneu_string_where_clause {
            fn from(s: &#pneu_str_name #pneu_string_type_generics) -> Self {
                let s = s.as_str().to_string();
                #self_construction
            }
        }

        impl #pneu_string_impl_generics std::str::FromStr for #pneu_string_name #pneu_string_type_generics #pneu_string_where_clause {
            type Err = <#pneu_str_name #pneu_string_type_generics as pneutype::Validate>::Error;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                <#pneu_str_name #pneu_string_type_generics as pneutype::Validate>::validate(s)?;
                let s = s.to_string();
                Ok(#self_construction)
            }
        }

        #serde_serialize_maybe

        impl #pneu_string_impl_generics std::borrow::ToOwned for #pneu_str_name #pneu_string_type_generics #pneu_string_where_clause {
            type Owned = #pneu_string_name #pneu_string_type_generics;
            fn to_owned(&self) -> Self::Owned {
                use std::ops::Deref;
                let s = self.deref().to_owned();
                #pneu_string_construction
            }
        }

        impl #pneu_string_impl_generics TryFrom<&str> for #pneu_string_name #pneu_string_type_generics #pneu_string_where_clause {
            type Error = <#pneu_str_name #pneu_string_type_generics as pneutype::Validate>::Error;
            fn try_from(s: &str) -> Result<Self, Self::Error> {
                <#pneu_str_name #pneu_string_type_generics as pneutype::Validate>::validate(s)?;
                let s = s.to_string();
                Ok(#self_construction)
            }
        }

        impl #pneu_string_impl_generics TryFrom<String> for #pneu_string_name #pneu_string_type_generics #pneu_string_where_clause {
            type Error = <#pneu_str_name #pneu_string_type_generics as pneutype::Validate>::Error;
            fn try_from(s: String) -> Result<Self, Self::Error> {
                <#pneu_str_name #pneu_string_type_generics as pneutype::Validate>::validate(s.as_str())?;
                unsafe { Ok(Self::new_unchecked(s)) }
            }
        }
    };

    // NOTE: This is for debugging the output of the proc macro.  `cargo expand` doesn't seem to actually capture
    // everything that goes wrong for some reason.  Note that it's useful to run `rustfmt` on the generated file.
    const DEBUG_OUTPUT: bool = false;
    if DEBUG_OUTPUT {
        let filename = format!("derive_pneu_string.{}.rs", pneu_string_name);
        let mut file = std::fs::File::create(filename.as_str())
            .expect(format!("Could not create file {:?}", filename).as_str());
        use std::io::Write;
        writeln!(file, "{}", output)
            .expect(format!("Could not write to file {:?}", filename).as_str());
    }

    output.into()
}

//
// proc_macro for creating a str-based newtype
//

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(pneu_str))]
struct PneuStrArguments {
    /// Specify true to derive an implementation of serde::Deserialize.  The `serde` crate must be imported into
    /// the crate in which this PneuStr is defined in order for this to work.  Using this attribute is optional,
    /// and a manual implementation of serde::Deserialize is of course possible.
    deserialize: bool,
    /// Specify true to derive an implementation of serde::Serialize.  The `serde` crate must be imported into
    /// the crate in which this PneuStr is defined in order for this to work.  Using this attribute is optional,
    /// and a manual implementation of serde::Serialize is of course possible.  However, in the case of a PneuStr
    /// with generics, this attribute must be used instead of derive(serde::Serialize) because of the presence of
    /// std::marker::PhantomData.
    serialize: bool,
    /// Optionally specify the `str`-valued field.  If not specified, then it will be "0" (i.e. for the ordinary
    /// case of `#[derive(pneutype::PneuStr)] #[repr(transparent)] pub struct ThingStr(str);`).  This attribute
    /// would be used in the case of a PneuStr having generics, e.g.
    /// `#[derive(pneutype::PneuStr)] #[pneu_str(str_field = "s")] #[repr(transparent)] pub struct ThingStr<T> { t: std::marker::PhantomData<T>, s: str }`
    str_field: Option<String>,
}

#[proc_macro_derive(PneuStr, attributes(pneu_str))]
pub fn derive_pneu_str(token_stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(token_stream);
    let pneu_str_arguments = PneuStrArguments::from_derive_input(&input).expect("Wrong arguments");
    let pneu_str_name = input.ident;
    let (pneu_str_impl_generics, pneu_str_type_generics, pneu_str_where_clause) =
        input.generics.split_for_impl();

    let str_field: syn::Expr =
        syn::parse_str(pneu_str_arguments.str_field.as_deref().unwrap_or("0")).unwrap();

    let serde_deserialize_maybe = if pneu_str_arguments.deserialize {
        // Create new lifetime parameters 'de and 'a
        let lifetime_a = syn::Lifetime::new("'a", proc_macro2::Span::call_site());
        let lifetime_de = syn::Lifetime::new("'de", proc_macro2::Span::call_site());

        let serde_deserialize_generics = {
            // Define the lifetimes with the correct relationship ('de: 'a)
            let lifetime_a_def = syn::LifetimeDef::new(lifetime_a.clone());
            let lifetime_de_def = syn::LifetimeDef {
                attrs: Vec::new(),
                lifetime: lifetime_de.clone(),
                colon_token: Some(syn::Token![:](proc_macro2::Span::call_site())),
                bounds: vec![lifetime_a.clone()].into_iter().collect(),
            };

            // Create a new Generics object with the new lifetimes added
            let mut new_generics = input.generics.clone();
            new_generics
                .params
                .insert(0, syn::GenericParam::Lifetime(lifetime_a_def));
            new_generics
                .params
                .insert(0, syn::GenericParam::Lifetime(lifetime_de_def));
            new_generics
        };
        let (
            serde_deserialize_impl_generics,
            _serde_deserialize_type_generics,
            _serde_deserialize_where_clause,
        ) = serde_deserialize_generics.split_for_impl();

        use quote::ToTokens;
        let (serde_deserialize_visitor, serde_deserialize_visitor_construction) =
            if pneu_str_type_generics.to_token_stream().is_empty() {
                (quote! { struct Visitor }, quote! { Visitor })
            } else {
                (
                    quote! {
                        struct Visitor #pneu_str_impl_generics(std::marker::PhantomData #pneu_str_type_generics) #pneu_str_where_clause
                    },
                    quote! { Visitor::#pneu_str_type_generics(std::marker::PhantomData::default()) },
                )
            };

        let serde_deserialize_visitor_generics = {
            // Define the lifetimes with the correct relationship ('de: 'a)
            let lifetime_a_def = syn::LifetimeDef::new(lifetime_a.clone());

            // Create a new Generics object with the new lifetimes added
            let mut new_generics = input.generics.clone();
            new_generics
                .params
                .insert(0, syn::GenericParam::Lifetime(lifetime_a_def));
            new_generics
        };
        let (
            serde_deserialize_visitor_impl_generics,
            _serde_deserialize_visitor_type_generics,
            _serde_deserialize_visitor_where_clause,
        ) = serde_deserialize_visitor_generics.split_for_impl();

        quote! {
            impl #serde_deserialize_impl_generics serde::Deserialize<#lifetime_de> for &#lifetime_a #pneu_str_name #pneu_str_type_generics #pneu_str_where_clause {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<#lifetime_de>,
                {
                    #serde_deserialize_visitor;

                    impl #serde_deserialize_visitor_impl_generics serde::de::Visitor<#lifetime_a> for Visitor #pneu_str_type_generics #pneu_str_where_clause {
                        type Value = &#lifetime_a #pneu_str_name #pneu_str_type_generics;

                        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                            formatter.write_str("a borrowed string")
                        }
                        fn visit_borrowed_str<E>(self, v: &#lifetime_a str) -> Result<Self::Value, E>
                        where
                            E: serde::de::Error,
                        {
                            #pneu_str_name::new_ref(v).map_err(serde::de::Error::custom)
                        }
                    }

                    deserializer.deserialize_str(#serde_deserialize_visitor_construction)
                }
            }
        }
    } else {
        quote! {}
    };

    let serde_serialize_maybe = if pneu_str_arguments.serialize {
        quote! {
            impl #pneu_str_impl_generics serde::Serialize for #pneu_str_name #pneu_str_type_generics #pneu_str_where_clause {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    serializer.serialize_str(&self.#str_field)
                }
            }
        }
    } else {
        quote! {}
    };

    let try_from_lifetime = syn::Lifetime::new("'s", proc_macro2::Span::call_site());
    let try_from_generics = {
        let try_from_lifetime_def = syn::LifetimeDef::new(try_from_lifetime.clone());

        // Create a new Generics object with the new lifetime added
        let mut new_generics = input.generics.clone();
        new_generics
            .params
            .insert(0, syn::GenericParam::Lifetime(try_from_lifetime_def));
        new_generics
    };
    let (try_from_impl_generics, _try_from_type_generics, _try_from_where_clause) =
        try_from_generics.split_for_impl();

    let output = quote! {
        impl #pneu_str_impl_generics #pneu_str_name #pneu_str_type_generics #pneu_str_where_clause {
            /// Validate the given str and wrap it as a reference to this PneuStr type.
            pub fn new_ref(s: &str) -> Result<&Self, <Self as pneutype::Validate>::Error> where Self: pneutype::Validate<Data = str> {
                use pneutype::Validate;
                Self::validate(s)?;
                unsafe { Ok(Self::new_ref_unchecked(s)) }
            }
            /// Unsafe: Wrap the given str as a reference to this PneuStr type without validating it.
            /// This requires the caller to guarantee validity.  However, a debug_assert! will be used
            /// to check the validity condition.  For a const version of this, see new_ref_unchecked_const.
            pub unsafe fn new_ref_unchecked(s: &str) -> &Self {
                debug_assert!(<Self as pneutype::Validate>::validate(s).is_ok(), "programmer error: new_ref_unchecked was passed invalid data");
                // See https://stackoverflow.com/questions/64977525/how-can-i-create-newtypes-for-an-unsized-type-and-its-owned-counterpart-like-s
                &*(s as *const str as *const Self)
            }
            /// Unsafe: Wrap the given str as a reference to this PneuStr type without validating it.
            /// This requires the caller to guarantee validity.  Because this is a const function, the
            /// validity condition can't be checked in a debug_assert! as it is in new_ref_unchecked.
            pub const unsafe fn new_ref_unchecked_const(s: &str) -> &Self {
                // See https://stackoverflow.com/questions/64977525/how-can-i-create-newtypes-for-an-unsized-type-and-its-owned-counterpart-like-s
                &*(s as *const str as *const Self)
            }
            /// Return the raw &str underlying this PneuStr.
            pub fn as_str(&self) -> &str {
                &self.#str_field
            }
        }

        impl #pneu_str_impl_generics std::convert::AsRef<str> for #pneu_str_name #pneu_str_type_generics {
            fn as_ref(&self) -> &str {
                Self::as_str(self)
            }
        }

        impl #pneu_str_impl_generics std::borrow::Borrow<str> for #pneu_str_name #pneu_str_type_generics {
            fn borrow(&self) -> &str {
                Self::as_str(self)
            }
        }

        impl #pneu_str_impl_generics std::ops::Deref for #pneu_str_name #pneu_str_type_generics {
            type Target = str;
            fn deref(&self) -> &Self::Target {
                Self::as_str(self)
            }
        }

        #serde_deserialize_maybe

        impl #pneu_str_impl_generics std::fmt::Display for #pneu_str_name #pneu_str_type_generics {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                Self::as_str(self).fmt(f)
            }
        }

        #serde_serialize_maybe

        impl #try_from_impl_generics TryFrom<&#try_from_lifetime str> for &#try_from_lifetime #pneu_str_name #pneu_str_type_generics {
            type Error = <#pneu_str_name #pneu_str_type_generics as pneutype::Validate>::Error;
            fn try_from(s: &#try_from_lifetime str) -> Result<Self, Self::Error> {
                #pneu_str_name::new_ref(s)
            }
        }
    };

    // NOTE: This is for debugging the output of the proc macro.  `cargo expand` doesn't seem to actually capture
    // everything that goes wrong for some reason.  Note that it's useful to run `rustfmt` on the generated file.
    const DEBUG_OUTPUT: bool = false;
    if DEBUG_OUTPUT {
        let filename = format!("derive_pneu_str.{}.rs", pneu_str_name);
        let mut file = std::fs::File::create(filename.as_str())
            .expect(format!("Could not create file {:?}", filename).as_str());
        use std::io::Write;
        writeln!(file, "{}", output)
            .expect(format!("Could not write to file {:?}", filename).as_str());
    }

    output.into()
}
