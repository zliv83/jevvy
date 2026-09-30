use std::env;

use crate::{
  constants::{API_KEY, BASE_URL},
  error::JevvyError,
  questions::Questions,
  Reply,
};

pub struct Jev {
  http:     reqwest::Client,
  api_key:  String,
  base_url: &'static str,
}

impl Jev {
  pub fn new(api_key: impl Into<String>) -> Self {
    Self {
      http:     reqwest::Client::new(),
      api_key:  api_key.into(),
      base_url: BASE_URL,
    }
  }

  pub fn from_env() -> Result<Self, env::VarError> {
    let api_key = env::var(API_KEY)?;
    Ok(Self::new(api_key))
  }

  #[cfg(feature = "dotenvy")]
  pub fn from_dotenv() -> Result<Self, JevvyError> {
    dotenvy::dotenv()?;
    Ok(Self::from_env()?)
  }

  pub async fn ask(&self, questions: &Questions) -> Result<Reply, JevvyError> {
    let response = self
      .http
      .post(self.base_url)
      .bearer_auth(&self.api_key)
      .json(questions)
      .send()
      .await?;

    let status = response.status();
    let body = response
      .bytes()
      .await?;

    if !status.is_success() {
      return Err(JevvyError::API {
        status,
        body: String::from_utf8_lossy(&body).into_owned(),
      });
    }
    let response = serde_json::from_slice(&body)?;

    Ok(response)
  }
}
