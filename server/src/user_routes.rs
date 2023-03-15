use axum::response::{IntoResponse, Response as AxumResponse};
use axum::{middleware, Router, routing};
use axum::http::{header, Request, StatusCode, HeaderMap};
use axum::middleware::Next;
use axum::routing::{get};
use chrono::{Duration, Utc};
use crate::encryption::{AuthConstructor, Keys, LoggedInUser};
use mongodb::{Client, Collection};
use mongodb::bson::{doc, Document};
use mongodb::options::ClientOptions;
use rand::Rng;
use ring::aead::{AES_256_GCM, BoundKey, Nonce, NONCE_LEN, NonceSequence, SealingKey, UnboundKey};
use crate::noncesequencehelper::{decrypt_and_decode, encrypt_and_encode, OneNonceSequence};

pub fn user_routes() -> Router {
  Router::new()
    .route("/user-1", get(basic_route))
    .route("/user-2", get(basic_route_2))
}


async fn middleware<B>(
  request: Request<B>,
  next: Next<B>
) -> Result<AxumResponse, StatusCode> {
  let headers = request.headers();

  let cookie_header = headers.get(header::COOKIE);

  let res = match cookie_header {
    Some(cookie) => {
      let mut cookie = cookie.to_str().unwrap_or_else(|e| {
        println!("Couldn't unwrap the cookie: {}", e);
        Err(StatusCode::UNAUTHORIZED).unwrap()
      });
      if let Some(i) = cookie.find("=") {
        cookie = &cookie[i + 1..];
      }

      // If we get here, the cookie is in place, we try to verify it.
      // This is the final defence for getting access to the user.
      match Keys::new().verify_user(cookie) {
        Ok(_) => Ok(next.run(request).await),
        Err(e) => {
          println!("Bailed on the user verification {}", e);
          Err(StatusCode::UNAUTHORIZED)
        }
      }
    }
    None => {
      println!("Bailed on the cookie presence");
      Err(StatusCode::UNAUTHORIZED)
    }
  };

  res
}

// all validation and stuff happens in the middleware. We can unwrap safely
// but should validate the token again, just for shits and giggles.
async fn basic_route(
  headers: HeaderMap,
) {

  let mut token
    = headers.get(header::COOKIE).unwrap().to_str().unwrap();

  if let Some(i) = token.find("=") {
    token = &token[i + 1..];
  }

  let LoggedInUser {
    username,
    uuid,
    exp: _ // token gets verified inside of the verify user.
  } = Keys::new().verify_user(token).unwrap();

  // get information from the database related to the user.
  // can be passwords or otherwise.


}

async fn basic_route_2() {

  let mut client_options =
    ClientOptions::parse("mongodb://localhost:27017").await.unwrap();

  client_options.app_name = Some("LessPM".to_string());
  let client = Client::with_options(client_options).unwrap();

  // for db_names in client.list_database_names(None, None).await.unwrap() {
  //   println!("{}", db_names);
  //
  // }

  let db = client.database("lesspm");

  let collection: Collection<Document> = db.collection("vault");

  let passwords = vec![
    doc! {
      "website": "https://google.com",
      "username": "havard@alacho.no",
      "password": "secret123"
    }, doc! {
      "website": "https://facebook.com",
      "username": "havard@alacho.no",
      "password": "secret123"
    }, doc! {
      "website": "https://google.com",
      "username": "havard@alacho.no",
      "password": "secret123"
    },
  ];

  // the size of my cred id is always 24 bytes.
  // That mean I need to pad it up with 8 bytes.


  // let random_padding: [u8; 8] = rand::thread_rng().gen();
  let random_padding: [u8; 32] = rand::thread_rng().gen();

  // let nonce_bytes: [u8; NONCE_LEN] = [0u8; NONCE_LEN];
  // let nonce = Nonce::assume_unique_for_key(nonce_bytes);
  //
  // let nonce_sequence = OneNonceSequence::new(nonce);

  let random_nonce: [u8; 12] = rand::thread_rng().gen();
  let random_nonce_two: [u8; 12] = rand::thread_rng().gen();

  // Details.
  // The nonce needs to be stored with the password,
  // otherwise you can't decrypt it
  // It acts as a second salt, I suppose.
  // It says you should through the nonce away, but you can't decrypt
  // it without the nonce, so ... What?

  let hello = encrypt_and_encode(
    &AES_256_GCM,
    "Hello, Wårld".to_string(),
    &random_padding,
    &random_nonce.to_vec(),
  ).unwrap();

  println!("{} ", &hello);
  let decrypted = decrypt_and_decode(
    &AES_256_GCM,
    hello,
    &random_padding,
    &random_nonce.to_vec()
  ).unwrap();

  println!("{}", decrypted);



  // let sealing_key = ring::aead::SealingKey::new(
  //   &aead::AES_256_GCM,
  //   &random_padding,
  // )



  // let something
  //   = ring::aead::SealingKey::;
  // let key = rand::

  // let result = collection.insert_many(passwords, None).await.unwrap();

  // dbg!(result);

}

