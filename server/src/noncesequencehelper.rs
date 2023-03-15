use base64::Engine;
use ring::aead;
use ring::aead::{NonceSequence, Nonce, NONCE_LEN, Algorithm, Aad, SealingKey, BoundKey, UnboundKey, OpeningKey};
use ring::error::Unspecified;

pub struct FixedNonceSequence {
  nonce: [u8; NONCE_LEN],
  increment: u64,
}

impl FixedNonceSequence {
  pub fn  new() -> Self {
    Self {
      nonce: [0u8; NONCE_LEN],
      increment: 0,
    }
  }
}

impl NonceSequence for FixedNonceSequence {
  fn advance(&mut self) -> Result<Nonce, Unspecified> {
    let mut nonce = self.nonce;
    self.increment = self.increment.checked_add(1).unwrap_or(0);
    nonce[4..].copy_from_slice(&self.increment.to_be_bytes());
    Ok(Nonce::assume_unique_for_key(nonce))
  }
}



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

pub fn encrypt_and_encode(
  algorithm: &'static Algorithm,
  input: String,
  key: &[u8],
  nonce: &Vec<u8>
) -> Result<String, String> {
  let n = aead::Nonce::try_assume_unique_for_key(nonce)
    .expect("Something");
    // Ok(n) => n,
    // Err(e) => eprintln!("Derp"),
  // };

  let mut raw = input.as_bytes().to_owned();

  match seal_with_key(algorithm, key, n, Aad::from(&[0; 0]), &mut raw) {
    Ok(_v) => _v,
    Err(e) => {
      println!("Bailed on seal {}", e);
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
