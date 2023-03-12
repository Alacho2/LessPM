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
  pub auth_state: PasskeyAuthentication,
  pub exp: usize,
}

// impl ClaimConstructor {
//   pub fn from_json(json: &str) -> Result<Self, Box<dyn std::error::Error>> {
//     let claim: ClaimConstructor = serde_json::from_str(json)?;
//     Ok(claim)
//   }
// }


impl Keys {
  pub fn new() -> Self {
    let encoding_key
      = EncodingKey::from_rsa_pem(include_bytes!("../keys/privatekey.pem"))
      .expect("Something went wrong with the encoding key");
    let decoding_key
      = DecodingKey::from_rsa_pem(include_bytes!("../keys/public.pem"))
      .expect("Something went wrong with the decoding key");
    Self {
      header: Header::new(Algorithm::RS256),
      encoding_key,
      decoding_key,
    }
  }

  // ☣️ THIS NEEDS TO BE ENCRYPTED. OTHERWISE THE CLIENT CAN FUCK WITH
  // THE STORAGE. We encrypt with RSA256. Suck on that, Alexander.
  pub fn token_claim(&self, claim: ClaimConstructor) -> String {
    encode(&self.header, &claim, &self.encoding_key)
      .expect("Something bad happened with the encoding")
  }

  pub fn verify_claim(&self, token: &str) -> Result<ClaimConstructor, Error> {
      decode(
        &token,
        &self.decoding_key,
        &Validation::new(Algorithm::RS256)
      ).map(|data: TokenData<ClaimConstructor>| data.claims)
  }

  pub fn token_auth(&self, claim: AuthConstructor) -> String {
    encode(&self.header, &claim, &self.encoding_key)
      .expect("Something bad happened during encoding")
  }

  pub fn verify_auth(&self, token: &str) -> Result<AuthConstructor, Error> {
    decode(
      &token,
      &self.decoding_key,
      &Validation::new(Algorithm::RS256)
    )
      .map(|data: TokenData<AuthConstructor>| data.claims)
  }
}
