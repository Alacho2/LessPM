use std::str::FromStr;
use axum::response::{IntoResponse, Response as AxumResponse};
use axum::{middleware, Router, routing};
use axum::extract::Path;
use axum::http::{header, Request, StatusCode, HeaderMap, HeaderValue};
use axum::middleware::Next;
use axum::routing::{get};
use chrono::{Duration, Utc};
use crate::encryption::{AuthConstructor, Keys, LoggedInUser};
use mongodb::{Client, Collection};
use mongodb::bson::{doc, Document};
use mongodb::bson::oid::ObjectId;
use mongodb::options::ClientOptions;
use rand::Rng;
use regex::Regex;
use ring::aead::{AES_256_GCM, BoundKey, Nonce, NONCE_LEN, NonceSequence, SealingKey, UnboundKey};
use crate::db_connection::DbConnection;
use crate::noncesequencehelper::{decrypt_and_decode, decrypt_and_retrieve, encrypt_and_encode, encrypt_and_store, OneNonceSequence};

pub fn user_routes() -> Router {
  Router::new()
    .route("/user-1", get(basic_route))
    .route("/user-2", get(basic_route_2))
    .route("/passwords", get(get_user_passwords))
    .route("/passwords/:id", get(get_password_in_clear_text))
}

// The process_cookie should return the logged in user OR ... Nothing?
pub fn process_cookie(
  header: Option<&HeaderValue>,
) -> Option<LoggedInUser> {
  let header = header?;

  let mut token = header.to_str().unwrap();

  if let Some(i) = token.find("=") {
    token = &token[i + 1..];
  }

  match Keys::new().verify_user(token) {
    Ok(verified) => Some(verified),
    Err(_) => None,
  }
}

fn is_valid_object_id(id: &str) -> bool {
  let re = Regex::new(r"^[0-9a-fA-F]{24}$").unwrap();
  re.is_match(id)
}


// This needs to be called from the context of authentication
// I should extract as much as I can into functions in there, so to create more reusability
async fn get_password_in_clear_text(
  headers: HeaderMap,
  Path(id): Path<String>
) -> impl IntoResponse {
  let cookie_header = headers.get(header::COOKIE);

  let processed_cookie = process_cookie(cookie_header);

  let err_response = axum::http::Response::builder()
    .status(StatusCode::UNAUTHORIZED)
    .body("".to_string())
    .unwrap();

  if processed_cookie.is_none() || !is_valid_object_id(&id) {
    return err_response;
  }

  let object_id_res = ObjectId::parse_str(id);
  if object_id_res.is_err() {
    return err_response;
  }


  let object_id = object_id_res.unwrap();
  let user = processed_cookie.unwrap();
  let username = user.username;
  decrypt_and_retrieve(username, object_id).await;
  // let optional_vault_entry = db.get_one("vault", object_id).await;

  axum::http::Response::builder()
    .status(StatusCode::OK)
    .body("".to_string())
    .unwrap()

}

async fn get_user_passwords(
  headers: HeaderMap
) -> impl IntoResponse {
  let cookie_header = headers.get(header::COOKIE);

  let error_response = axum::http::Response::builder()
    .status(StatusCode::UNAUTHORIZED)
    .body("".to_string())
    .unwrap();

  // If the cookie header isn't there
  if cookie_header == None {
    return error_response;
  }

  let mut token = cookie_header.unwrap().to_str().unwrap();

  if let Some(i) = token.find("=") {
    token = &token[i + 1..];
  }

  let logged_in_user_res = Keys::new().verify_user(token);

  if logged_in_user_res.is_err() {
    return error_response;
  }

  let LoggedInUser {
    username,
    uuid: _,
    exp: _
  } = logged_in_user_res.unwrap();

  let db = DbConnection::new().await;

  let passwords
      = db.get_passwords("vault", &username).await;

  if passwords.is_err() {
    return error_response;
  }

  let result = passwords.unwrap();

  axum::http::Response::builder()
      .status(StatusCode::OK)
      .body(serde_json::to_string(&result).unwrap().into())
      .unwrap()
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
      // This is the final defence for getting access to the user information.
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

  let val_vec: Vec<u8>
    = vec![64, 225, 160, 67, 171, 21, 68, 138, 110, 51, 44, 48, 170, 224, 63,
           253, 29, 226, 11, 132, 73, 203, 198, 179];

  // encrypt_and_store("secret123", val_vec).await;



  // let random_padding: [u8; 8] = rand::thread_rng().gen();
  // let random_padding: [u8; 32] = rand::thread_rng().gen();

  // let nonce_bytes: [u8; NONCE_LEN] = [0u8; NONCE_LEN];
  // let nonce = Nonce::assume_unique_for_key(nonce_bytes);
  //
  // let nonce_sequence = OneNonceSequence::new(nonce);

  // let random_nonce: [u8; 12] = rand::thread_rng().gen();
  // let random_nonce_two: [u8; 12] = rand::thread_rng().gen();

  // Details.
  // The nonce needs to be stored with the password,
  // otherwise you can't decrypt it
  // It acts as a second salt, I suppose.
  // It says you should through the nonce away, but you can't decrypt
  // it without the nonce, so ... What?

  // let hello = encrypt_and_encode(
  //   &AES_256_GCM,
  //   "Hello, Wårld".to_string(),
  //   &random_padding,
  //   &random_nonce.to_vec(),
  // ).unwrap();

  // println!("{} ", &hello);
  // let decrypted = decrypt_and_decode(
  //   &AES_256_GCM,
  //   hello,
  //   &random_padding,
  //   &random_nonce.to_vec()
  // ).unwrap();

  // println!("{}", decrypted);



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

