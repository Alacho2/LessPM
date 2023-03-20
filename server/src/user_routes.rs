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
    .route("/passwords", get(get_user_passwords))
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
    username:_,
    uuid,
    exp: _
  } = user_logged_in.unwrap();

  let db = DbConnection::new().await;

  let passwords
      = db.get_passwords("vault", &uuid.to_string()).await;

  if passwords.is_err() {
    return error_response;
  }

  let result = passwords.unwrap();

  axum::http::Response::builder()
      .status(StatusCode::OK)
      .body(serde_json::to_string(&result).unwrap().into())
      .unwrap()
}

