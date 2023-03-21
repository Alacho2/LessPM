use std::fmt::{Display, Formatter};
use jsonwebtoken::{Algorithm, decode, DecodingKey, encode, EncodingKey, Header, TokenData, Validation};
use jsonwebtoken::errors::Error;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Sha512};
use webauthn_rs::prelude::{PasskeyAuthentication, PasskeyRegistration, Uuid};
use crate::noncesequencehelper::{decrypt_with_key, encrypt_with_key};

const PEPPER_EXTRACTOR_LENGTH: usize = 16;

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

pub struct EncryptionProcess {
  pub salt: [u8; 8],
  pub nonce: [u8; 12],
  pub key_padding: Vec<u8>,
  pub random_padding: [u8; 12],
  pub base64: String,
}

impl EncryptionProcess {
  // this function should ONLY return the values needed to store. NOT store.
  pub fn start(validator_vec: &Vec<u8>, input: &str) -> EncryptionProcess {
    let (cred_id_as_arr, bits, random_padding)
      = EncryptionProcess::generate_416bit_arr_of_vec(validator_vec);
    let pretended_salt = EncryptionProcess::generate_a_salt();

    let mut key_for_aes = [0u8; 32];
    pbkdf2::pbkdf2_hmac::<Sha512>(&cred_id_as_arr, &pretended_salt, 4096, &mut key_for_aes);

    let nonce: [u8; 12] = rand::thread_rng().gen();

    let base64 = encrypt_with_key(
      input,
      &key_for_aes,
      &nonce,
    ).unwrap();

    EncryptionProcess {
      salt: pretended_salt,
      key_padding: bits,
      random_padding,
      nonce,
      base64,
    }
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


  // MAX 24 bytes of the validator
  // At LEAST 12 bytes of padding
  // 16 bytes of pepper.
  fn generate_416bit_arr_of_vec(validator_vec: &Vec<u8>) -> ([u8; 52], Vec<u8>, [u8; 12]) {
    let pepper = std::env::var("PEPPER").unwrap();
    let pepper_as_bytes = pepper.as_bytes();

    // Check the length of the validator
    let vec_len = validator_vec.len();

    let length_of_key = 24;

    // Gives us no MORE than 24 bytes.
    let initial_bytes
      = if vec_len >= length_of_key { length_of_key } else { vec_len };

    let mut arr = [0u8; 52];

    // Take the necessary parts off of the validator
    for i in 0..initial_bytes {
      arr[i] = validator_vec[i];
    }

    let remaining_bytes_to_reach_desired_key =
      if initial_bytes >= length_of_key { 0 } else { 24 - vec_len };

    let mut bits: Vec<u8> = Vec::new();
    // Collect the remaining bytes needed to reach a key length of 24.
    for i in 0..remaining_bytes_to_reach_desired_key {
      let num: u8 = rand::thread_rng().gen();
      bits.push(num);
      arr[i + initial_bytes] = num;
    }

    // add 12 bytes of random padding.
    let where_does_padding_go = initial_bytes + remaining_bytes_to_reach_desired_key;
    let random_padding: [u8; 12] = rand::thread_rng().gen();
    for i in 0..random_padding.len() {
      arr[i + where_does_padding_go] = random_padding[i];
    }

    let where_should_pepper_go
      = initial_bytes + random_padding.len() + remaining_bytes_to_reach_desired_key;

    // add 16 bytes from the pepper.
    // Too much and we risk creating too large of the key to be known.
    for i in 0..PEPPER_EXTRACTOR_LENGTH {
      arr[i + where_should_pepper_go] = pepper_as_bytes[i];
    }

    dbg!(arr);

    (arr, bits, random_padding)
  }

  fn recreate_key(
    validator_vec: &Vec<u8>,
    process: &EncryptionProcess
  ) -> [u8; 52] {
    let pepper = std::env::var("PEPPER").unwrap();
    let pepper_as_bytes = pepper.as_bytes();

    let mut arr = [0u8; 52];

    let vec_len = validator_vec.len();
    let length_of_key = 24;

    let initial_bytes = if vec_len >= length_of_key { length_of_key } else { vec_len };

    let bits = &process.key_padding;

    // validator part of the key
    for i in 0..initial_bytes {
      arr[i] = validator_vec[i];
    }

    for i in 0..bits.len() {
      arr[i + initial_bytes] = bits[i];
    }


    let where_does_padding_go = initial_bytes + bits.len();
    let random_padding_len = process.random_padding.len();

    // padding of the key
    for i in 0..random_padding_len {
      arr[i + where_does_padding_go] = process.random_padding[i];
    }

    let where_to_put_the_pepper
      = where_does_padding_go + random_padding_len;

    // pepper part of the key
    for i in 0..PEPPER_EXTRACTOR_LENGTH {
      arr[i + where_to_put_the_pepper] = pepper_as_bytes[i];
    }

    dbg!(arr);

    arr
  }
}