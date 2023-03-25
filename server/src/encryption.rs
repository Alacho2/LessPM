use std::fmt::{Display, Formatter};
use argon2::{Argon2, ParamsBuilder, Version, Algorithm as ArgonAlgorithm};
use base64::Engine;
use jsonwebtoken::{Algorithm, decode, DecodingKey, encode, EncodingKey, Header, TokenData, Validation};
use jsonwebtoken::errors::Error;
use rand::Rng;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey};
use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::{PasskeyAuthentication, PasskeyRegistration, Uuid};
use crate::noncesequencehelper::{decrypt_with_key, encrypt_with_key};

const PEPPER_EXTRACTOR_LENGTH: usize = 16;
const NON_UNIQUE_NONCE: [u8; 12]
  = [197, 107, 7, 215, 179, 237, 89, 104, 200, 204, 34, 243];
const NON_UNIQUE_AES_KEY: [u8; 32]
  = [127, 119, 136, 168, 251, 211, 76, 164, 56, 22, 195, 233, 140, 6, 150, 236, 232, 160, 8, 226, 96, 222, 9, 116, 137, 212, 146, 35, 28, 45, 245, 195];

pub struct Keys {
  header: Header,
  validator: Validation,
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


// ☣️ JWT NEEDS TO BE ENCRYPTED. OTHERWISE THE CLIENT CAN FUCK WITH
// THE STORAGE. We encrypt with RSA512 and encrypt with AES256. Suck on that, Alexander.
impl Keys {
  pub fn new() -> Self {
    let encoding_key
      = EncodingKey::from_rsa_pem(include_bytes!("../keys/privatekey.pem"))
      .expect("Something went wrong with the encoding key");
    let decoding_key
      = DecodingKey::from_rsa_pem(include_bytes!("../keys/public.pem"))
      .expect("Something went wrong with the decoding key");
    let algorithm = Algorithm::PS512;
    Self {
      validator: Validation::new(algorithm),
      header: Header::new(algorithm),
      encoding_key,
      decoding_key,
    }
  }
  pub fn tokenize_claim(&self, claim: ClaimConstructor) -> String {
    match encode(&self.header, &claim, &self.encoding_key) {
      Ok(base64) => JwtEncryption::encrypt(base64),
      Err(e) => {
        eprintln!("Claim: Base64 encoding failed: {}", e);
        String::from("")
      }
    }
  }

  pub fn verify_claim(&self, token: &str) -> Result<ClaimConstructor, Error> {
    let base64_decrypted = JwtEncryption::decrypt(token);
    decode::<ClaimConstructor>(&base64_decrypted, &self.decoding_key, &self.validator)
      .map(|data: TokenData<ClaimConstructor>| data.claims)
  }

  pub fn tokenize_auth(&self, claim: AuthConstructor) -> String {
    match encode(&self.header, &claim, &self.encoding_key) {
      Ok(base64) => JwtEncryption::encrypt(base64),
      Err(e) => {
        eprintln!("Auth: Base64 encoding failed: {}", e);
        String::from("")
      }
    }
  }

  pub fn verify_auth(&self, token: &str) -> Result<AuthConstructor, Error> {
    let base64_decrypted = JwtEncryption::decrypt(token);
    decode(&base64_decrypted, &self.decoding_key, &self.validator)
      .map(|data: TokenData<AuthConstructor>| data.claims)
  }

  pub fn tokenize_user(&self, claim: LoggedInUser) -> String {
    match encode(&self.header, &claim, &self.encoding_key) {
      Ok(base64) => JwtEncryption::encrypt(base64),
      Err(e) => {
        eprintln!("User: Base64 encoding failed: {}", e);
        String::from("")
      }
    }
  }

  pub fn verify_user(&self, token: &str) -> Result<LoggedInUser, Error> {
    let base64_decrypted = JwtEncryption::decrypt(token);
    decode(&base64_decrypted, &self.decoding_key, &self.validator)
      .map(|data: TokenData<LoggedInUser>| data.claims)
  }
}

pub struct EncryptionProcess {
  pub salt: [u8; 12],
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
    let key_for_aes
      = EncryptionProcess::hash_construct_helper(cred_id_as_arr, pretended_salt);

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
    process: EncryptionProcess
  ) -> String {
    let cred_id_as_arr
      = EncryptionProcess::recreate_key(&validator_vec, &process);

    let pretended_salt = process.salt;
    let key_for_aes = EncryptionProcess::hash_construct_helper(cred_id_as_arr, pretended_salt);

    let nonce = process.nonce;

    let res = decrypt_with_key(process.base64, &key_for_aes, &nonce).unwrap();
    res
  }

  fn generate_a_salt() -> [u8; 12] {
    let salt: [u8; 12] = rand::thread_rng().gen();
    salt
  }

  fn hash_construct_helper(arr: [u8; 52], pretended_salt: [u8; 12]) -> [u8; 32] {
    // You gone goofed up if you didn't configure these in an .env file
    let memory: u32 = std::env::var("MEMORY").unwrap().parse().unwrap();
    let iterations: u32 = std::env::var("ITERATIONS").unwrap().parse().unwrap();
    let parallels_config: u32 = std::env::var("PARALLELS").unwrap().parse().unwrap();

    // Save the user of this function from their own disaster.
    let parallels = if parallels_config > 255 { 255 } else { parallels_config };

    // Decrease the memory usage in the config file if you want to decrease the time needed to hash
    let params = ParamsBuilder::new()
      .m_cost(1024 * memory)
      .t_cost(iterations)
      .p_cost(parallels)
      .build()
      .unwrap();

    let algo = ArgonAlgorithm::default();
    let version = Version::default();
    let argon2 = Argon2::new(algo, version, params);
    let mut key_for_aes = [0u8; 32];
    argon2.hash_password_into(&arr, &pretended_salt, &mut key_for_aes).unwrap();
    key_for_aes
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

    let remaining_bytes_to_reach_desired_length =
      if initial_bytes >= length_of_key { 0 } else { 24 - vec_len };

    let mut bits: Vec<u8> = Vec::new();
    // Collect the remaining bytes needed to reach a key length of 24.
    for i in 0..remaining_bytes_to_reach_desired_length {
      let num: u8 = rand::thread_rng().gen();
      bits.push(num);
      arr[i + initial_bytes] = num;
    }

    // add 12 bytes of random padding.
    let padding_pos = initial_bytes + remaining_bytes_to_reach_desired_length;
    let random_padding: [u8; 12] = rand::thread_rng().gen();
    for i in 0..random_padding.len() {
      arr[i + padding_pos] = random_padding[i];
    }

    let pepper_pos
      = initial_bytes + random_padding.len() + remaining_bytes_to_reach_desired_length;

    // add 16 bytes from the pepper.
    // Too much and we risk creating too large of the key to be known.
    for i in 0..PEPPER_EXTRACTOR_LENGTH {
      arr[i + pepper_pos] = pepper_as_bytes[i];
    }

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

    let padding_pos = initial_bytes + bits.len();
    let random_padding_len = process.random_padding.len();

    // padding of the key
    for i in 0..random_padding_len {
      arr[i + padding_pos] = process.random_padding[i];
    }

    let pepper_pos
      = padding_pos + random_padding_len;

    // pepper part of the key
    for i in 0..PEPPER_EXTRACTOR_LENGTH {
      arr[i + pepper_pos] = pepper_as_bytes[i];
    }

    arr
  }
}

pub struct JwtEncryption {
  nonce: Nonce,
  less_safe_key: LessSafeKey,
  aad: Aad<[u8; 8]>
}

impl JwtEncryption {

  fn new() -> Self {
    let algorithm = &ring::aead::AES_256_GCM;
    let unbound_key
      = UnboundKey::new(algorithm, &NON_UNIQUE_AES_KEY).unwrap();
    Self {
      nonce: Nonce::assume_unique_for_key(NON_UNIQUE_NONCE),
      less_safe_key: LessSafeKey::new(unbound_key),
      aad: Aad::from([65, 18, 243, 102, 187, 59, 94, 105])
    }
  }

  pub fn encrypt(base64: String) -> String {
    let JwtEncryption { nonce, less_safe_key, aad } = JwtEncryption::new();
    let mut in_out = base64.as_bytes().to_owned();
    less_safe_key.seal_in_place_append_tag(nonce, aad, &mut in_out).unwrap();
    base64::engine::general_purpose::STANDARD.encode(in_out)
  }

  pub fn decrypt(base64: &str) -> String {
    let mut decoded
      = base64::engine::general_purpose::STANDARD.decode(base64).unwrap();
    let JwtEncryption { nonce, less_safe_key, aad } = JwtEncryption::new();
    let decrypted= less_safe_key.open_within(nonce, aad, &mut decoded, std::ops::RangeFrom{start: 0}).unwrap();
    String::from_utf8(decrypted.to_vec()).unwrap()
  }
}