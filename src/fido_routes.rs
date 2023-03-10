use std::convert::Infallible;
use std::fmt::{Debug, Formatter};
use axum::{Router, routing, extract::State, Json, http};
use u2f::protocol::{Challenge, U2f};
use u2f::register::Registration;
use std::sync::Mutex;
use axum::body::{Body, BoxBody};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response as AxumResponse};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm, TokenData};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Timelike, Utc};
use jsonwebtoken::errors::Error;
use u2f::messages::U2fRegisterRequest;

use webauthn_rs;
use webauthn_rs::prelude::{PasskeyRegistration, RegisterPublicKeyCredential, Uuid, WebauthnError, WebauthnResult};
use crate::app_state::AppState;
use crate::encryption::{ClaimConstructor, Keys};
use crate::response::Response;

mod fido;



// lazy_static! {
//   static ref REGISTRATIONS: Mutex<Vec<Registration>> = {
//     let registrations: Mutex<Vec<Registration>> = Mutex::new(vec![]);
//     registrations
//   };
// }



// There should probably be some sort of endpoint to SIGN IN to the server.
// but for now, we just get everything up and run.

pub fn api_routes(state: AppState) -> Router {
  Router::new()
    .route("/start_registration", routing::post(start_registration))
    .route("/finish_registration", routing::post(finish_registration))
    .route("/sign_response", routing::post(sign_response))
    .route("/register_response", routing::post(register_response))
    .with_state(state)
}


#[derive(Debug, Serialize, Deserialize)]
struct User {
  name: String,
}

async fn start_registration(
  state: State<AppState>,
  Json(body): Json<User>
) -> Result<impl IntoResponse, &'static str> {
  println!("Derp");

  let username = body.name;
  let user_id = Uuid::new_v4();



  let res = match state.authn.start_passkey_registration(
    user_id,
    &username,
    &username,
    None
  ) {
    Ok((ccr, reg_state)) => {
      let two_minutes = (Utc::now() + Duration::minutes(2)).timestamp();
      let claim = ClaimConstructor {
        user_id,
        username,
        reg_state,
        exp: two_minutes as usize,
      };
      let token = Keys::new().token(claim);

      let default_response_builder: AxumResponse<Body> =
        Response::response_builder(StatusCode::OK, token)
          .body(serde_json::to_string(&ccr).unwrap().into())
          .unwrap();

      default_response_builder;

      Json(ccr)

      // Grab the username, user id and reg_state and encode it in a token
      // println!("Registration successful, {}", token);
    }
    Err(e) => {
      dbg!("start_registration -> {:?}", e);
      return Err("Unknown error");
    }
  };
  Ok(res)
}

async fn finish_registration(
  state: State<AppState>,
  Json(reg): Json<RegisterPublicKeyCredential>
)  {

}

// POST request
async fn sign_response(state: State<AppState>) {

}

// POST request
async fn register_response(state: State<AppState>) {

}
