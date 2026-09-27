//! The grammar every attribute shares: on described thing.
//!
//! Inside the parentheses is a plain string, or named parts.
//!
//! ```text
//! #[describe("Order status, delivery, cancellation, or returns")]
//! #[describe(what = "Charges, invoices, refunds", examples = ["I was charged twice"])]
//! ```
//!
//! The first becomes `Entry::Text`, the second `Entry::Object` with those keys.
//! A value is a string, a list in `[...]`, or nested parts in `{...}`, so any
//! shape the docs show can be written in an attribute.

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
  braced, bracketed,
  ext::IdentExt,
  parse::{Parse, ParseStream},
  punctuated::Punctuated,
  spanned::Spanned,
  token, Attribute, Error, Ident, LitStr, Token,
};

/// One `key = value` part of a structured description.
pub struct Pair {
  /// The key, e.g. `what`. Sent as a JSON key.
  pub key:   Ident,
  /// The value: a string, a list, or nested parts.
  pub value: Value,
}

/// A value in a description. The JSON shapes the API accepts, as tokens.
pub enum Value {
  /// `"..."`
  Text(LitStr),
  /// `["...", "..."]`
  List(Vec<Value>),
  /// `{ what = "...", examples = [...] }`
  Object(Vec<Pair>),
}

/// Everything inside an attribute's parentheses: an optional leading value,
/// then any number of `key = value` parts.
///
/// Callers `take` their reserved keys (a Noul's `when_true` and `when_false`)
/// and turn the rest into one [`Value`].
pub struct Body {
  /// Where the attribute is, for errors.
  span:  Span,
  /// The leading value, if any: `#[describe("...")]`.
  lead:  Option<Value>,
  /// The named parts, in order written.
  pairs: Vec<Pair>,
}

impl Body {
  /// Parses the contents of `attr`'s parentheses.
  pub fn parse(attr: &Attribute) -> syn::Result<Self> {
    let span = attr.span();
    attr.parse_args_with(|input: ParseStream| {
      // A leading value is anything that isn't `key =`.
      let lead = if input.is_empty() || input.peek2(Token![=]) {
        None
      } else {
        let value = input.parse()?;
        if !input.is_empty() {
          input.parse::<Token![,]>()?;
        }
        Some(value)
      };
      let pairs = parse_pairs(input)?;
      Ok(Body { span, lead, pairs })
    })
  }

  /// Pulls out a reserved `key = value`, if it was written.
  pub fn take(&mut self, key: &str) -> Option<Value> {
    let at = self
      .pairs
      .iter()
      .position(|pair| pair.key == key)?;
    Some(
      self
        .pairs
        .remove(at)
        .value,
    )
  }

  /// What's left, s one value. `what` names the thing for the error.
  /// e.g. "the question for `team`".
  pub fn into_value(self, what: &str) -> syn::Result<Value> {
    match (
      self.lead,
      self
        .pairs
        .is_empty(),
    ) {
      | (Some(lead), true) => Ok(lead),
      | (None, false) => Ok(Value::Object(self.pairs)),
      | (Some(lead), false) => Err(Error::new(
        lead.span(),
        format!("{what} is either one value or named parts, not both"),
      )),
      | (None, true) => Err(Error::new(
        self.span,
        format!("{what} is missing: write a string, or named parts like `what = \"...\"`"),
      )),
    }
  }
}

/// `key = value, key = value, ...` up to the end of `input.
/// A key written twice is an error at the second one.
fn parse_pairs(input: ParseStream) -> syn::Result<Vec<Pair>> {
  let pairs: Vec<Pair> = Punctuated::<Pair, Token![,]>::parse_terminated(input)?
    .into_iter()
    .collect();
  for (i, pair) in pairs
    .iter()
    .enumerate()
  {
    if pairs[..i]
      .iter()
      .any(|earlier| earlier.key == pair.key)
    {
      return Err(Error::new(
        pair
          .key
          .span(),
        format!("`{}` is written twice", pair.key),
      ));
    }
  }
  Ok(pairs)
}

impl Parse for Pair {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    // `parse_any` also accepts keywords so `type = "string"` works
    let key = input.call(Ident::parse_any)?;
    input.parse::<Token![=]>()?;
    let value = input.parse()?;
    Ok(Pair { key, value })
  }
}

impl Parse for Value {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    if input.peek(LitStr) {
      return Ok(Value::Text(input.parse()?));
    }
    if input.peek(token::Bracket) {
      let inner;
      bracketed!(inner in input);
      let items = Punctuated::<Value, Token![,]>::parse_terminated(&inner)?;
      return Ok(Value::List(
        items
          .into_iter()
          .collect(),
      ));
    }
    if input.peek(token::Brace) {
      let inner;
      braced!(inner in input);
      return Ok(Value::Object(parse_pairs(&inner)?));
    }

    Err(input.error("expected a string, a list `[\"...\"]`, or parts `{ what = \"...\" }`"))
  }
}

impl Value {
  /// Where this value was written.
  pub fn span(&self) -> Span {
    match self {
      | Value::Text(text) => text.span(),
      | Value::List(items) => items
        .first()
        .map_or_else(Span::call_site, Value::span),
      | Value::Object(pairs) => pairs
        .first()
        .map_or_else(Span::call_site, |pair| {
          pair
            .key
            .span()
        }),
    }
  }

  /// This value as an `Entry` expression: `Entry::from("...")` for text,
  /// `Entry::from(json!(...))` for a list or parts.
  pub fn entry(&self) -> TokenStream {
    match self {
      | Value::Text(text) => quote! { ::jevvy::types::Entry::from(#text)},
      | other => quote! {
        ::jevvy::types::Entry::from(::jevvy::__private::serde_json::json!(#other))
      },
    }
  }
}

/// A value in `json!` syntax, for nesting inside another value.
impl ToTokens for Value {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    tokens.extend(match self {
      | Value::Text(text) => quote! { #text },
      | Value::List(items) => quote! { [ #(#items),* ] },
      | Value::Object(pairs) => quote! { { #(#pairs),* } },
    });
  }
}

/// `"key": value` in `json!` syntax.
impl ToTokens for Pair {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    // `unraw` turns `r#type` into `type`, so the JSON key is clean.
    let key = self
      .key
      .unraw()
      .to_string();

    let value = &self.value;
    tokens.extend(quote! { #key: #value });
  }
}
