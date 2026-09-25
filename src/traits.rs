use std::hash::Hash;

/// An enum whose variants are the options of a Choice question.
///
/// `Copy + Eq + Hash` lets a variant be a key in `Choice::probabilities`.
pub trait Options: Copy + Eq + Hash + 'static {
  /// Every variant, in the order the model should see them.
  const ALL: &'static [Self];

  /// The name sent to the API, e.g. `"networking"`.
  fn name(&self) -> &'static str;

  /// What this option means. `None` sends no description.
  fn describe(&self) -> Option<&'static str> {
    None
  }

  /// Turns a name from the API back into a variant.
  fn from_name(name: &str) -> Option<Self> {
    Self::ALL
      .iter()
      .copied()
      .find(|option| option.name() == name)
  }
}
