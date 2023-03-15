use std::fmt::{Display, Formatter, write};
use jsonwebtoken::{Algorithm, decode, DecodingKey, encode, EncodingKey, Header, TokenData, Validation};
use jsonwebtoken::errors::Error;
use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::{PasskeyAuthentication, PasskeyRegistration, Uuid};

pub struct Keys {
  header: Header,
  encoding_key: EncodingKey,
  decoding_key: DecodingKey,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClaimConstructor {
  pub user_id: Uuid,
  pub username: String,
  pub reg_state: PasskeyRegistration,
  pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthConstructor {
  pub user_id: Uuid,
  pub username: String,
  pub auth_state: PasskeyAuthentication,
  pub exp: usize,
}

#[derive(Serialize, Deserialize)]
pub struct LoggedInUser {
  pub username: String,
  pub uuid: Uuid,
  pub exp: usize,
}

impl Display for LoggedInUser {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "User({} {})", self.username, self.uuid)
  }
}

impl Keys {
  pub fn new() -> Self {
    let encoding_key
      = EncodingKey::from_rsa_pem(include_bytes!("../keys/privatekey.pem"))
      .expect("Something went wrong with the encoding key");
    let decoding_key
      = DecodingKey::from_rsa_pem(include_bytes!("../keys/public.pem"))
      .expect("Something went wrong with the decoding key");
    Self {
      header: Header::new(Algorithm::PS512),
      encoding_key,
      decoding_key,
    }
  }

  // ☣️ THIS NEEDS TO BE ENCRYPTED. OTHERWISE THE CLIENT CAN FUCK WITH
  // THE STORAGE. We encrypt with RSA256. Suck on that, Alexander.
  pub fn tokenize_claim(&self, claim: ClaimConstructor) -> String {
    encode(&self.header, &claim, &self.encoding_key)
      .expect("Something bad happened with the encoding")
  }

  pub fn verify_claim(&self, token: &str) -> Result<ClaimConstructor, Error> {
      decode(
        &token,
        &self.decoding_key,
        &Validation::new(Algorithm::PS512)
      ).map(|data: TokenData<ClaimConstructor>| data.claims)
  }

  pub fn tokenize_auth(&self, claim: AuthConstructor) -> String {
    encode(&self.header, &claim, &self.encoding_key)
      .expect("Something bad happened during encoding")
  }

  pub fn verify_auth(&self, token: &str) -> Result<AuthConstructor, Error> {
    decode(
      &token,
      &self.decoding_key,
      &Validation::new(Algorithm::PS512)
    ).map(|data: TokenData<AuthConstructor>| data.claims)
  }

  // TODO(Håvard): You need to document that you'd rather use a private key for
  // each thing.

  pub fn tokenize_user(&self, claim: LoggedInUser) -> String {
    encode(&self.header, &claim, &self.encoding_key)
      .expect("Can't tokenize the user")
  }

  pub fn verify_user(&self, token: &str) -> Result<LoggedInUser, Error> {
    decode(
      &token,
      &self.decoding_key,
      &Validation::new(Algorithm::PS512)
    ).map(|data: TokenData<LoggedInUser>| data.claims)
  }
}
