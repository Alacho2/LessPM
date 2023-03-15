use base64::Engine;
use ring::aead::{NonceSequence, Nonce, NONCE_LEN, Algorithm, Aad, SealingKey, BoundKey, UnboundKey, OpeningKey};
use ring::error::Unspecified;
use rand::Rng;

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

pub fn encrypt_and_store(
  input: String,
  validator_vec: Vec<u8>
) {

  let size_of_validator_key = std::mem::size_of_val("hello");

  dbg!(size_of_validator_key);

  // Check whether the size of value is smaller than

  // let a = vec![0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
  // let b = &a; // b: &Vec<u8>
  // let c: &[u8] = &a; // c: &[u8]

  // What if we just make a type fo this?
  let mut random_padding: Vec<u8> = Vec::new();
  let another_bits_attempts: [u8; 32] = if validator_vec.len() >= 24 {

    // Grab 192 bits off of the key
    let mut bits_192: [u8; 24] = [0u8; 24];
    for i in 0..24 {
      bits_192[i] = validator_vec[i];
    }

    // let bits_192: [u8; 24] = validator_vec.try_into().unwrap();
    // we are missing 8 bytes

    // Create a random padding to fill up the rest of the key.
    for _ in 1..8 {
      let num: u8 = rand::thread_rng().gen();
      random_padding.push(num);
    }

    // Construct the remaining 64 bites to create an AESkey
    let mut aes_key = [0u8; 32];
    for i in 0..bits_192.len() {
      aes_key[i] = bits_192[i];
    }

    for i in 0..random_padding.len() {
      aes_key[i + bits_192.len()] = random_padding[i];
    }

    let nonce = rand::thread_rng().gen();

    let value = KeyHelper {
      nonce,
      random_padding,
      key: aes_key,
    };


    // let to_an_arr = random_padding.try_into().unwrap();

    // Now you have the bits and the padding
    // random_padding: [u8; 8] = rand::thread_rng().gen();
    [0u8; 32]
  } else {
    [0u8; 32]
  };

  // let bites_24_small_from_validator: [u8; 24] = validator_vec.try_into().unwrap();
  // let random_padding: [u8; 12] = rand::thread_rng().gen();
}

pub fn encrypt_and_encode(
  algorithm: &'static Algorithm,
  input: String,
  key: &[u8],
  nonce: &Vec<u8>
) -> Result<String, String> {
  let n = Nonce::try_assume_unique_for_key(nonce)
    .expect("Something");

  let mut some_vec: Vec<u8> = Vec::new();

  for _ in 0..26 {
    let num: u8 = rand::thread_rng().gen();
    some_vec.push(num);
  }

  encrypt_and_store("Hello, World!".to_string(), some_vec);

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
