use std::sync::Arc;
use axum::response::{IntoResponse, Response as AxumResponse};
use axum::{async_trait, body, Json, middleware, Router};
use axum::body::{BoxBody, Bytes, Full};
use axum::extract::{FromRequest, Path};
use axum::http::{header, Request, StatusCode, HeaderMap, HeaderValue};
use axum::middleware::Next;
use axum::routing::{get};
use jsonwebtoken::jwk::KeyOperations::Verify;
use crate::encryption::{AuthConstructor, ClaimConstructor, Keys, LoggedInUser};
use mongodb::bson::oid::ObjectId;
use regex::Regex;
use crate::db_connection::DbConnection;
use crate::noncesequencehelper::{decrypt_and_decode, decrypt_and_retrieve, decrypt_with_key, encrypt_and_encode, encrypt_and_store, OneNonceSequence};

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

pub fn process_auth_token(
  header: Option<&HeaderValue>
) -> Option<AuthConstructor> {
  let header = header?;

  let mut token = header.to_str().unwrap();

  if let Some(i) = token.find(" ") {
    token = &token[i + 1..];
  }

  match Keys::new().verify_auth(&token) {
    Ok(verified) => Some(verified),
    Err(_) => None,
  }
}

pub fn process_claim_token(
  header: Option<&HeaderValue>
) -> Option<ClaimConstructor> {
  let header = header?;

  let mut token = header.to_str().unwrap();
  if let Some(i) = token.find(" ") {
    token = &token[i + 1..];
  }

  match Keys::new().verify_claim(&token) {
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

  let user_logged_in = process_cookie(cookie_header);

  if user_logged_in.is_none() {
    return error_response;
  }

  let LoggedInUser {
    username,
    uuid: _,
    exp: _
  } = user_logged_in.unwrap();

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
  // we should take in the key

  // let val_vec: Vec<u8>
  //   = vec![64, 225, 160, 67, 171, 21, 68, 138, 110, 51, 44, 48, 170, 224, 63,
  //          253, 29, 226, 11, 132, 73, 203, 198, 179];
  // let res = EncryptionProcess::start(&val_vec, "Hello, Wårld!");
  //
  //
  // let something = EncryptionProcess::end(&val_vec, res);
  //
  // dbg!(something);


  // the res can now be stored in the db

  /*

  let mut random_vec = [0u8; 24];

  for i in 0..val_vec.len() {
    random_vec[i] = val_vec[i];
  }

  // for i in 0..8 {
  //   let num = rand::thread_rng().gen();
  //   random_vec[i + val_vec.len()] = num;
  // }
  //
  let random_padding: [u8; 8] = rand::thread_rng().gen();

  // plus the random values
  let mut key = [0u8; 32];
  pbkdf2::pbkdf2_hmac::<Sha256>(&random_vec, &random_padding, 4096, &mut key);

  dbg!(key);
   */


  /*
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
 */
}

