use std::fmt::Error;
use base64::Engine;
use ring::aead::{NonceSequence, Nonce, NONCE_LEN, Algorithm, Aad, SealingKey, BoundKey, UnboundKey, OpeningKey, AES_256_GCM};
use ring::error::Unspecified;
use rand::Rng;
use crate::db_connection::{DbConnection, VaultEntry};

pub struct OneNonceSequence(Option<Nonce>);

impl OneNonceSequence {
  pub fn new(nonce: Nonce) -> Self {
    Self(Some(nonce))
  }
}

impl NonceSequence for OneNonceSequence {
  fn advance(&mut self) -> Result<Nonce, Unspecified> {
    self.0.take().ok_or(Unspecified)
  }
}

struct KeyHelper {
  nonce: [u8; 12],
  random_padding: Vec<u8>,
  key: [u8; 32],
}



pub async fn encrypt_and_store(
  username: String,
  input: &str,
  website: String,
  validator_vec: Vec<u8>
) -> anyhow::Result<()> {
  let KeyHelper {
    key,
    nonce,
    random_padding
  } = generate_aes_key(&validator_vec);

  let algorithm = &AES_256_GCM;

  let base64_encoding= encrypt_and_encode(
    algorithm,
    input.to_string(),
    &key,
    &nonce.to_vec()
  )?;

  let db = DbConnection::new().await;

  let help = VaultEntry {
    username,
    password: base64_encoding.to_string(),
    website,
    nonce,
    random_padding
  };
  db.insert_one("vault", help).await;



  // println!("{}", base64_encoding);
  //
  // let decrypted = decrypt_and_decode(
  //   algorithm,
  //   base64_encoding,
  //   &key,
  //   &nonce.to_vec()
  // ).unwrap();
  //
  // println!("{}", decrypted);

  // Now that things are encrypted, we can store that in the db

  Ok(())
}

// Takes the validator vec
fn generate_aes_key(validator_vec: &Vec<u8>) -> KeyHelper {
  let mut aes_key = [0u8; 32];
  let mut random_padding: Vec<u8> = Vec::new();
  let nonce: [u8; 12] = rand::thread_rng().gen();

  let vec_len = validator_vec.len();

  let remaining_bytes_helper
    = if vec_len >= 24 { 8 } else { 32 - vec_len };
  let initial_bytes_helper
    = if vec_len >= 24 { 24 } else { vec_len };

  for i in 0..initial_bytes_helper {
    aes_key[i] = validator_vec[i];
  }

  // The random part of the key
  for i in 0..remaining_bytes_helper {
    let num: u8 = rand::thread_rng().gen();
    aes_key[i + initial_bytes_helper] = num;
    random_padding.push(num);
  }

  KeyHelper {
    random_padding,
    nonce,
    key: aes_key,
  }
}

pub fn encrypt_and_encode(
  algorithm: &'static Algorithm,
  input: String,
  key: &[u8],
  nonce: &Vec<u8>
) -> anyhow::Result<String> {
  let n = Nonce::try_assume_unique_for_key(nonce)
    .expect("Something");

  let mut raw = input.as_bytes().to_owned();

  match seal_with_key(algorithm, key, n, Aad::from(&[0; 0]), &mut raw) {
    Ok(_v) => _v,
    Err(e) => {
      println!("Bailed on seal {}", e);
      return Err(anyhow::Error::msg("Hello"));
    }
  };

  Ok(base64::engine::general_purpose::STANDARD.encode(raw))
}

pub fn decrypt_and_decode(
  algorithm: &'static Algorithm,
  input: String,
  key: &[u8],
  nonce: &Vec<u8>
) -> Result<String, String> {
  let mut raw = match base64::engine::general_purpose::STANDARD.decode(input) {
    Ok(r) => r,
    Err(e) => {
      println!("Bailed on decode {}", e);
      Err("Bailed on decode".to_string()).unwrap()
    }
  };

  let n = match Nonce::try_assume_unique_for_key(nonce) {
    Ok(n) => n,
    Err(e) => {
      println!("Bailed on Nonce {}", e);
      Err("Bailed on Nonce".to_string()).unwrap()
    }
  };

  let res = match open_with_key(
    algorithm,
    &key,
    n,
    Aad::from(&[0; 0]),
    &mut raw,
    std::ops::RangeFrom{start: 0}
  ) {
    Ok(v) => v,
    Err(e) => {
      println!("Bailed on opening {}", e);
      Err("Bailed on opening".to_string()).unwrap()
    }
  };



  let res = match String::from_utf8(res.to_vec()) {
    Ok(value) => value,
    Err(e) => {
      println!("Bailed on string conversation");
      Err("Bailed on utf8".to_string()).unwrap()
    }
  };

  Ok(res)
}

fn seal_with_key(
  algorithm: &'static Algorithm,
  key: &[u8],
  nonce: Nonce,
  aad: Aad<&[u8]>,
  in_out: &mut Vec<u8>
) -> Result<(), Unspecified> {
  let mut s_key: SealingKey<OneNonceSequence> = make_key(algorithm, key, nonce);
  s_key.seal_in_place_append_tag(aad, in_out)
}

fn make_key<K: BoundKey<OneNonceSequence>>(
  algorithm: &'static Algorithm,
  key: &[u8],
  nonce: Nonce
) -> K {
  let key = UnboundKey::new(algorithm, key).unwrap();
  let nonce_sequence = OneNonceSequence::new(nonce);
  K::new(key, nonce_sequence)
}

fn open_with_key<'a>(
  algorithm: &'static Algorithm,
  key: &[u8],
  nonce: Nonce,
  aad: Aad<&[u8]>,
  in_out: &'a mut [u8],
  ciphertext_and_tag: std::ops::RangeFrom<usize>,
) -> Result<&'a mut [u8], Unspecified> {
  let mut o_key: OpeningKey<OneNonceSequence> = make_key(algorithm, key, nonce);
  o_key.open_within(aad, in_out, ciphertext_and_tag)
  // o_key.open_in_place(aad, in_out)
}
