use jevvy::{
  client::Jevvy,
  traits::{Levels, Options, Rubric},
  types::{
    error::JevvyError,
    questions::{Question, Questions},
    response::Response,
    typed::{Choice, Score},
  },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Area {
  Networking,
  Serde,
  Docs,
}

impl Options for Area {
  const ALL: &'static [Self] = &[Area::Networking, Area::Serde, Area::Docs];

  fn name(&self) -> &'static str {
    match self {
      | Area::Networking => "networking",
      | Area::Serde => "serde",
      | Area::Docs => "docs",
    }
  }

  fn describe(&self) -> Option<&'static str> {
    match self {
      | Area::Networking => Some("HTTP client, retries, timeouts"),
      | Area::Serde => Some("Request/response types, JSON shape"),
      | Area::Docs => Some("README, examples, docs.rs"),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Severity {
  Trivial,
  Annoying,
  Blocker,
}

impl Levels for Severity {
  const ALL: &'static [Self] = &[Severity::Trivial, Severity::Annoying, Severity::Blocker];

  fn describe(&self) -> &'static str {
    match self {
      | Severity::Trivial => "Trivial: cosmetic, nobody is slowed down",
      | Severity::Annoying => "Annoying: there's a workaround, but it hurts",
      | Severity::Blocker => "Blocker: users can't get their work done",
    }
  }
}

/// What the bot wants to know about each GitHub issue.
#[derive(Debug)]
struct Triage {
  is_bug:    f64,
  has_repro: f64,
  area:      Choice<Area>,
  severity:  Score<Severity>,
}

impl Rubric for Triage {
  fn questions() -> Questions {
    // Questions is just an IndexMap<String, Question>, so it can be built
    // from an array of (key, question) pairs. Order is kept.
    Questions::from([
      (
        "is_bug".into(),
        Question::noul("Is this a bug report, not a question or a feature request?"),
      ),
      (
        "has_repro".into(),
        Question::noul("Does it include steps to reproduce the problem?"),
      ),
      (
        "area".into(),
        Area::question("Which part of the project does this concern?"),
      ),
      (
        "severity".into(),
        Severity::question("How badly does this affect users?"),
      ),
    ])
  }

  fn from_response(res: &Response) -> Result<Self, JevvyError> {
    Ok(Triage {
      is_bug:    res.noul("is_bug")?,
      has_repro: res.noul("has_repro")?,
      // 1st `?`: answer missing or wrong kind. 2nd `?`: the bouncer said no.
      // try_into() comes free with TryFrom. The field type picks the target.
      area:      res
        .choice("area")?
        .try_into()?,
      severity:  res
        .score("severity")?
        .try_into()?,
    })
  }
}

fn labels_for(t: &Triage) -> Vec<&'static str> {
  let mut labels = Vec::new();

  if t.is_bug > 0.8 {
    labels.push("bug");
    if t.has_repro < 0.5 {
      labels.push("needs-repro");
    }
  }

  match t
    .area
    .confidence
  {
    | 0.8.. => labels.push(
      t.area
        .value
        .name(),
    ),
    | 0.5..0.8 => labels.push("needs-triage"),
    | _ => println!("not sure about the ara, pinging a maintainer"),
  }

  if t
    .severity
    .level
    == Severity::Blocker
  {
    labels.push("p0")
  }

  labels
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  dotenvy::dotenv()?;
  let jevvy = Jevvy::from_env()?;

  let issue = serde_json::json!({
    "title": "Requests hang forever when the server never responds",
    "body": "Calling ask() against a slow server never returns. There's no timeout. \
             Steps: point TYPESAFE_BASE_URL at a serer tha acctpts the connection \
             but never replies, then call ask().",
  });

  let real: Triage = jevvy
    .ask(issue)
    .await?;

  println!(
    "is_bug: {:.2}, has_repro: {:.2}",
    real.is_bug, real.has_repro
  );
  println!(
    "area: {:?} ({:.0}% sure)",
    real
      .area
      .value,
    real
      .area
      .confidence
      * 100.0
  );
  println!(
    "severity: {:?} (raw {:.2})",
    real
      .severity
      .level,
    real
      .severity
      .raw
  );
  println!("labels: {:?}", labels_for(&real));

  Ok(())
}
