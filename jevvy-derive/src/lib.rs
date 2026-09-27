//! The derive macros for `jevvy`: `Options`, `Levels` and `Form`.
//!
//! Each one writes exactly what a hand-written impl says.
//! Use them through `jevvy`, which re-exports them next to the traits they
//! implement: `use jevvy::traits::{Form, Levels, Options};`.

mod described;
mod enums;
mod options;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// An enum whose variants are a Choice's options.
///
/// Per variant: `#[name("...")]` (optional; the default is the variant in
/// snake_case) and `#[describe(...)]` (optional; a string, or named parts).
#[proc_macro_derive(Options, attributes(name, describe))]
pub fn derive_options(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  options::derive(input)
    // A parse error now becomes a `compile_error!` which will point at the code.
    .unwrap_or_else(syn::Error::into_compile_error)
    .into()
}
