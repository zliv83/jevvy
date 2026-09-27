//! `#[derive(Options)]`: an enum whose variants are a Choice's options.
//!
//! ```rust
//! #[derive(Clone, Copy, PartialEq, Hash, Options)]
//! enum Team {
//!   #[describe(what = "Charges, invoices, refunds", examples = ["I was charged twice"])]
//!   Billing,
//!   #[describe("Order status, deliver, cancellation, or returns")]
//!   Orders,
//!   #[describe("none_of_the_above")]
//!   Other,
//!  }
//! ```
//!
//! Writes `ALL` in declaration order, `name()` (snake_case of the variant unless `#[name]`
//! says otherwise), and `describe()`, when any variant has a `#[describe]`. Without one, the
//! trait's default `None` stands.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, LitStr};

use crate::{
  described::Body,
  enums::{find_attr, snake_case, unit_variants},
};

pub fn derive(input: DeriveInput) -> syn::Result<TokenStream> {
  let name = &input.ident;
  let variants = unit_variants(&input, "Options")?;

  // Three parallel lists, one entry per variant, sipped by `#(...)*` below.
  let mut idents = Vec::new();
  let mut names = Vec::new();
  let mut describes = Vec::new();
  for variant in &variants {
    let ident = &variant.ident;
    let wire_names = match find_attr(&variant.attrs, "name")? {
      | Some(attr) => attr.parse_args::<LitStr>()?,
      | None => LitStr::new(&snake_case(&ident.to_string()), ident.span()),
    };
    let describe = match find_attr(&variant.attrs, "describe")? {
      | Some(attr) => {
        let value = Body::parse(attr)?.into_value(&format!("the description of `{ident}`"))?;
        let entry = value.entry();
        quote! { ::core::option::Option::Some(#entry) }
      }
      | None => quote! { ::core::option::Option::None },
    };
    idents.push(ident);
    names.push(wire_names);
    describes.push(describe);
  }

  // Only write `describe` when something is described. Otherwise the traits
  // default (`None`) is correct.
  let any_described = variants
    .iter()
    .any(|variant| {
      variant
        .attrs
        .iter()
        .any(|attr| {
          attr
            .path()
            .is_ident("describe")
        })
    });
  let describe_fn = any_described.then(|| {
    quote! {
      fn describe(&self) -> ::core::option::Option<::jevvy::types::Entry> {
        match self {
          #( | #name::#idents => #describes, )*
        }
      }
    }
  });

  Ok(quote! {
    impl ::jevvy::traits::Options for #name {
      const ALL: &'static [Self] = &[ #( #name::#idents ),* ];

      fn name(&self) -> &'static str {
        match self {
          #( | #name::#idents => #names, )*
        }
      }

      // Beside `name`, not inside it: both are methods of the impl.
      #describe_fn
    }
  })
}
