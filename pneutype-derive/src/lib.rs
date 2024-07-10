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
}

#[proc_macro_derive(PneuString, attributes(pneu_string))]
pub fn derive_pneu_string(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input);
    let pneu_string_arguments =
        PneuStringArguments::from_derive_input(&input).expect("Wrong arguments");
    let pneu_string_name = input.ident;

    let pneu_str_name: syn::Ident = syn::parse_str(&pneu_string_arguments.borrow).unwrap();

    let output = quote! {
        impl #pneu_string_name {
            /// Validate and construct this PneuString from a String.
            pub fn new(s: String) -> Result<Self, <#pneu_str_name as pneutype::Validate>::Error> {
                use pneutype::Validate;
                #pneu_str_name::validate(s.as_str())?;
                unsafe { Ok(Self::new_unchecked(s)) }
            }
            /// Unsafe: Construct this PneuString where the input is already guaranteed (by the caller) to be valid.
            unsafe fn new_unchecked(s: String) -> Self {
                Self(s)
            }
            /// Return a &str to the underlying String.
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
            /// Dissolve this instance and take the underlying String.
            pub fn into_string(self) -> String {
                self.0
            }
        }

        impl std::borrow::Borrow<#pneu_str_name> for #pneu_string_name {
            fn borrow(&self) -> &#pneu_str_name {
                use std::ops::Deref;
                self.deref()
            }
        }

        impl std::ops::Deref for #pneu_string_name {
            type Target = #pneu_str_name;
            fn deref(&self) -> &Self::Target {
                unsafe { #pneu_str_name::new_ref_unchecked(self.0.as_str()) }
            }
        }

        impl std::fmt::Display for #pneu_string_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                self.0.fmt(f)
            }
        }

        impl std::str::FromStr for #pneu_string_name {
            type Err = <#pneu_str_name as pneutype::Validate>::Error;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                use pneutype::Validate;
                #pneu_str_name::validate(s)?;
                Ok(Self(s.to_string()))
            }
        }

        impl std::borrow::ToOwned for #pneu_str_name {
            type Owned = #pneu_string_name;
            fn to_owned(&self) -> Self::Owned {
                #pneu_string_name(self.0.to_owned())
            }
        }

        impl TryFrom<&str> for #pneu_string_name {
            type Error = <#pneu_str_name as pneutype::Validate>::Error;
            fn try_from(s: &str) -> Result<Self, Self::Error> {
                use pneutype::Validate;
                #pneu_str_name::validate(s)?;
                Ok(Self(s.to_string()))
            }
        }

        impl TryFrom<String> for #pneu_string_name {
            type Error = <#pneu_str_name as pneutype::Validate>::Error;
            fn try_from(s: String) -> Result<Self, Self::Error> {
                Self::new(s)
            }
        }

        impl From<&#pneu_str_name> for #pneu_string_name {
            fn from(s: &#pneu_str_name) -> Self {
                Self(s.as_str().to_string())
            }
        }
    };

    output.into()
}

//
// proc_macro for creating a str-based newtype
//

#[proc_macro_derive(PneuStr)]
pub fn derive_pneu_str(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: syn::DeriveInput = syn::parse_macro_input!(input);
    let pneu_str_name = input.ident;

    let output = quote! {
        impl #pneu_str_name {
            /// Validate the given str and wrap it as a reference to this PneuStr type.
            pub fn new_ref(s: &str) -> Result<&Self, <Self as pneutype::Validate>::Error> where Self: pneutype::Validate {
                use pneutype::Validate;
                Self::validate(s)?;
                unsafe { Ok(Self::new_ref_unchecked(s)) }
            }
            /// Unsafe: Wrap the given str as a reference to this PneuStr type without validating it.
            /// This requires the caller to guarantee validity.
            unsafe fn new_ref_unchecked(s: &str) -> &Self {
                // See https://stackoverflow.com/questions/64977525/how-can-i-create-newtypes-for-an-unsized-type-and-its-owned-counterpart-like-s
                &*(s as *const str as *const #pneu_str_name)
            }
            /// Return the raw &str underlying this PneuStr.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::ops::Deref for #pneu_str_name {
            type Target = str;
            fn deref(&self) -> &Self::Target {
                self.as_str()
            }
        }

        impl std::fmt::Display for #pneu_str_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                self.0.fmt(f)
            }
        }

        impl<'s> TryFrom<&'s str> for &'s #pneu_str_name {
            type Error = <#pneu_str_name as pneutype::Validate>::Error;
            fn try_from(s: &'s str) -> Result<Self, Self::Error> {
                #pneu_str_name::new_ref(s)
            }
        }
    };

    output.into()
}
