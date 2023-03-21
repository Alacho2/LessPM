use axum::response::IntoResponse;
use axum::{Router};
use axum::http::{header, StatusCode, HeaderMap, HeaderValue};
use axum::routing::{get};
use crate::encryption::{AuthConstructor, ClaimConstructor, Keys, LoggedInUser};
use crate::db_connection::DbConnection;

pub fn user_routes() -> Router {
  Router::new()
    // .route("/test", get(something))
    .route("/passwords", get(get_user_passwords))
    .route("/authenticated", get(is_authenticated))
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

async fn is_authenticated(
  headers: HeaderMap
) -> StatusCode {
  let cookie_header = headers.get(header::COOKIE);
  match process_cookie(cookie_header) {
    Some(_) => StatusCode::OK,
    None => StatusCode::UNAUTHORIZED,
  }
}

