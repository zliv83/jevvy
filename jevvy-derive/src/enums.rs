//! What `Options` and `Levels` share: an enum of unit variants, and
//! the attributes on each one.

use syn::{Attribute, Data, DeriveInput, Error, Fields, Variant};

/// The enum's variants, each checked to be a plain name with no fields.
///
/// `derive` names the macro for the error: "Options" or "Levels".
pub fn unit_variants<'a>(input: &'a DeriveInput, derive: &str) -> syn::Result<Vec<&'a Variant>> {
  let Data::Enum(data) = &input.data else {
    return Err(Error::new_spanned(
      &input.ident,
      format!(
        "`{derive}` derives on an enum, one variant per {}",
        thing(derive)
      ),
    ));
  };
  data
    .variants
    .iter()
    .map(|variant| match variant.fields {
      | Fields::Unit => Ok(variant),
      | _ => Err(Error::new_spanned(
        variant,
        format!(
          "{} `{}` can't carry data: it's a name the model picks",
          thing(derive),
          variant.ident
        ),
      )),
    })
    .collect()
}

/// "option" for Options, "level" for Levels.
fn thing(derive: &str) -> &'static str {
  if derive == "Options" {
    "option"
  } else {
    "level"
  }
}

// The one attribute called `name` on `attrs`, if there is one.
// Two of them is an error at the second.
pub fn find_attr<'a>(attrs: &'a [Attribute], name: &str) -> syn::Result<Option<&'a Attribute>> {
  let mut found = None;
  for attr in attrs
    .iter()
    .filter(|attr| {
      attr
        .path()
        .is_ident(name)
    })
  {
    if found.is_some() {
      return Err(Error::new_spanned(
        attr,
        format!("`#[{name}]` is written twice. Pick one."),
      ));
    }
    found = Some(attr);
  }
  Ok(found)
}

/// `NoneOfTheAbove` to `none_of_the_above`, `HTTPError` to 'http_error`.
pub fn snake_case(name: &str) -> String {
  let chars: Vec<char> = name
    .chars()
    .collect();
  let mut out = String::with_capacity(name.len() + 4);
  for (i, &c) in chars
    .iter()
    .enumerate()
  {
    if c.is_uppercase() && i > 0 {
      let after_lower = chars[i - 1].is_lowercase() || chars[i - 1].is_ascii_digit();
      // `HTTPError`: the `E` starts a word because an `r` follows it.
      let starts_word = chars
        .get(i + 1)
        .is_some_and(|next| next.is_lowercase());
      if after_lower || starts_word {
        out.push('_');
      }
    }
    out.extend(c.to_lowercase());
  }
  out
}

#[cfg(test)]
mod tests {
  use super::snake_case;

  #[test]
  fn snake_cases() {
    assert_eq!(snake_case("Billing"), "billing");
    assert_eq!(snake_case("NoneOfTheAbove"), "none_of_the_above");
    assert_eq!(snake_case("HTTPError"), "http_error");
    assert_eq!(snake_case("Net30"), "net30");
    assert_eq!(snake_case("V2Beta"), "v2_beta");
  }
}
