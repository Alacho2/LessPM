use std::fmt::{Display, Formatter};
use jsonwebtoken::{Algorithm, decode, DecodingKey, encode, EncodingKey, Header, TokenData, Validation};
use jsonwebtoken::errors::Error;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Sha512};
use webauthn_rs::prelude::{PasskeyAuthentication, PasskeyRegistration, Uuid};
use crate::noncesequencehelper::{decrypt_with_key, encrypt_with_key};

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

const A_TEMP_PEPPER: &str
  = "g%255Fb6!@uC9K2g2L!cq@bEj#3A9VRn&TkjyS^fxAGMEstAZdQg&gDbbkez!e#XB@";

pub struct EncryptionProcess {
  pub salt: [u8; 8],
  pub nonce: [u8; 12],
  pub key_padding: Vec<u8>,
  pub base64: String,
}

impl EncryptionProcess {
  // this function should ONLY return the values needed to store. NOT store.
  pub fn start(validator_vec: &Vec<u8>, input: &str) -> EncryptionProcess {
    let cred_id_as_arr
      = EncryptionProcess::generate_320bit_arr_of_vec(validator_vec);
    let pretended_salt = EncryptionProcess::generate_a_salt();

    let mut key_for_aes = [0u8; 32];
    pbkdf2::pbkdf2_hmac::<Sha512>(&cred_id_as_arr.0, &pretended_salt, 4096, &mut key_for_aes);

    let nonce: [u8; 12] = rand::thread_rng().gen();

    let base64 = encrypt_with_key(
      input,
      &key_for_aes,
      &nonce,
    ).unwrap();

    EncryptionProcess {
      salt: pretended_salt,
      key_padding: cred_id_as_arr.1,
      nonce,
      base64,
    }
  }

  fn recreate_key(
    validator_vec: &Vec<u8>,
    whatever: &EncryptionProcess
  ) -> [u8; 40] {
    let pepper = std::env::var("PEPPER").unwrap();
    let pepper_as_bytes = pepper.as_bytes();

    let mut arr = [0u8; 40];

    // validator part of the key
    for i in 0..validator_vec.len() {
      arr[i] = validator_vec[i];
    }

    // padding of the key
    for i in 0..whatever.key_padding.len() {
      arr[i + validator_vec.len()] = whatever.key_padding[i];
    }

    let where_to_put_the_pepper
      = validator_vec.len() + whatever.key_padding.len();

    // pepper part of the key
    for i in 0..=15 {
      arr[i + where_to_put_the_pepper] = pepper_as_bytes[i]
    }

    arr
  }

  pub fn end(
    validator_vec: &Vec<u8>,
    whatever: EncryptionProcess
  ) -> String {
    let cred_id_as_arr
      = EncryptionProcess::recreate_key(&validator_vec, &whatever);

    let pretended_salt = whatever.salt;

    let mut key_for_aes = [0u8; 32];
    pbkdf2::pbkdf2_hmac::<Sha512>(&cred_id_as_arr, &pretended_salt, 4096, &mut key_for_aes);

    let nonce = whatever.nonce;

    let res = decrypt_with_key(whatever.base64, &key_for_aes, &nonce).unwrap();
    res
  }

  fn generate_a_salt() -> [u8; 8] {
    let salt: [u8; 8] = rand::thread_rng().gen();
    salt
  }

  fn generate_320bit_arr_of_vec(validator_vec: &Vec<u8>) -> ([u8; 40], Vec<u8>) {
    let pepper = std::env::var("PEPPER").unwrap();
    let pepper_as_bytes = pepper.as_bytes();

    // Check the length of the validator
    let vec_len = validator_vec.len();

    let length_of_key = 24;

    let initial_bytes_helper
      = if vec_len >= length_of_key { length_of_key } else { vec_len };
    let remaining_bytes_helper
      = if initial_bytes_helper >= length_of_key { 0 } else { length_of_key - vec_len };

    let mut arr = [0u8; 40];

    for i in 0..initial_bytes_helper {
      arr[i] = validator_vec[i];
    }

    let mut random_vec: Vec<u8> = Vec::new();
    // we probably need to return this as well somehow.
    for i in 0..remaining_bytes_helper {
      let num: u8 = rand::thread_rng().gen();
      random_vec.push(num);
      arr[i + vec_len] = num;
    }

    // Last but not least, add the pepper to the key
    // add 15 bytes from the pepper.
    // Too much and we risk creating too large of the key to be known.
    for i in 0..=15 {
      arr[i + length_of_key] = pepper_as_bytes[i];
    }


    // I need the random padding of the key

    (arr, random_vec)
  }
}