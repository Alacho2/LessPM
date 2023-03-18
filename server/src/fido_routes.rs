use std::borrow::Cow;
use std::fmt::{Debug};
use axum::{Router, routing, extract::State, Json, middleware, response};
use axum::body::{Body};
use axum::http::{
  header,
  HeaderMap,
  HeaderValue,
  Request,
  StatusCode
};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response as AxumResponse};
use axum::routing::get;
use serde::{Deserialize, Serialize};
use chrono::{Duration, Local, Utc};
use tokio::runtime::Runtime;

use webauthn_rs;
use webauthn_rs::prelude::{
  CredentialID,
  PublicKeyCredential,
  RegisterPublicKeyCredential,
  Uuid,
};
use crate::app_state::AppState;
use crate::encryption::{AuthConstructor, ClaimConstructor, Keys, LoggedInUser};
use crate::noncesequencehelper::{encrypt_and_encode, encrypt_and_store};
use crate::response::Response;

async fn test_route(
  headers: HeaderMap,
  state: State<AppState>,
  // Json(body): Json<User>
) -> Result<StatusCode, StatusCode> {

  let mut cookie = headers.get(header::COOKIE).unwrap().to_str().unwrap();

  if let Some(i) = cookie.find('=') {
    cookie = &cookie[i + 1..];
  }

  let user = Keys::new().verify_user(&cookie);

  match user {
    Ok(verified) => {
      // dbg!(verified);
      // We call the next request.
      Ok(StatusCode::OK)
    },
    Err(e) => {
      eprintln!("{} derp", e);
      Err(StatusCode::UNAUTHORIZED)
    },
  }



  // dbg!(cookie);


  // for (name, value) in headers.iter() {
  //   print!("{}", name.to_string());
  //   println!("{}", value.to_str().unwrap());
  // }

  // let cookie
  //   = format!("{}={}; HttpOnly; SameSite=Strict; Path=/", "token", "henlo");

  // let resp: AxumResponse<Body> = axum::http::Response::builder()
  //   .status(StatusCode::OK)
  //   .header(header::CONTENT_TYPE, "application/json")
  //   .header(header::COOKIE, HeaderValue::from_str(cookie.as_str()).unwrap())
  //   .header(header::ACCESS_CONTROL_EXPOSE_HEADERS, header::COOKIE)
  //   .body("".to_string().into())
  //   .unwrap();
  // resp
  // dbg!(body);
  // StatusCode::OK
}

#[derive(Deserialize, Serialize)]
struct ValidationOfPassword {
  credentials: PublicKeyCredential,
  #[serde(rename = "userData")]
  user_data: UserData,
}

#[derive(Debug, Deserialize, Serialize)]
struct UserData {
  website: String,
  username: String,
  password: String,
}

pub fn api_routes(state: AppState) -> Router {
  Router::new()
      .route("/test", get(test_route))
      .route("/finish_registration",
             routing::post(finish_registration)
                 .layer(middleware::from_fn(register_middleware))
      )
      .route("/start_registration", routing::post(start_registration))
      .route("/start_authentication",
             routing::post(start_authentication)
      )
      .route("/finish_authentication",
             routing::post(finish_authentication)
                 .layer(middleware::from_fn(auth_middleware))
      )
      .route("/start_password_creation", routing::post(start_password_creation))
      .route("/end_password_creation", routing::post(end_password_creation))
      .with_state(state)
}


#[derive(Debug, Serialize, Deserialize)]
pub struct User {
  pub name: String,
}

async fn start_registration(
  state: State<AppState>,
  Json(body): Json<User>
) -> impl IntoResponse {

  let username = body.name;
  let user_unique_id = {
    let users_guard = state.users.lock().await;
    users_guard
        .name_to_id
        .get(&username)
        .copied()
        .unwrap_or_else(Uuid::new_v4)
  };

  let exclude_credentials: Option<Vec<CredentialID>> = {
    let users_guard = state.users.lock().await;
    users_guard
        .keys
        .get(&user_unique_id)
        .map(|keys| keys.iter().map(|sk| sk.cred_id().clone()).collect())
  };

  // dbg!(&exclude_credentials);

  let res = match state.authn.start_passkey_registration(
    user_unique_id,
    &username,
    &username,
    None
    // exclude_credentials
  ) {
    Ok((ccr, reg_state)) => {
      // we use one minute to align with the default in the webauthn-lib
      let one_minute = (Utc::now() + Duration::minutes(1)).timestamp();
      let claim = ClaimConstructor {
        user_id: user_unique_id,
        username,
        reg_state,
        exp: one_minute as usize,
      };

      let token = Keys::new().tokenize_claim(claim);

      let default_response_builder: AxumResponse<Body> =
          Response::response_builder(StatusCode::OK, token)
              .body(serde_json::to_string(&ccr).unwrap().into())
              .unwrap();

      // dbg!(&default_response_builder);

      default_response_builder
    }
    Err(e) => {
      dbg!("start_registration -> {:?}", e);
      AxumResponse::builder()
          .status(StatusCode::BAD_REQUEST)
          .body("".to_string().into())
          .unwrap()

    }
  };
  res
}

async fn register_middleware<B>(
  request: Request<B>,
  next: Next<B>
) -> Result<AxumResponse, StatusCode> {
  let headers = request.headers();

  let token = headers.get(header::AUTHORIZATION);

  match token {
    Some(token) => {
      let mut token = token.to_str().unwrap();

      // Remove the Bearer-part of the string
      if let Some(i) = token.find(' ') {
        token = &token[i + 1..];
      }

      let claim =
          Keys::new().verify_claim(token);

      // TEMP UNTIL STORAGE. If the temp doesn't return an error, it's valid.
      match claim {
        Ok(_) => {
          Ok(next.run(request).await)
        }
        Err(e) => {
          println!("{}", e);
          Err(StatusCode::UNAUTHORIZED)
        }
      }
    }
    None => {
      Err(StatusCode::UNAUTHORIZED)
    },
  }
}

async fn finish_registration(
  header: HeaderMap,
  state: State<AppState>,
  Json(reg): Json<RegisterPublicKeyCredential>
) -> StatusCode {

  let mut token = header.get(header::AUTHORIZATION).unwrap().to_str().unwrap();

  if let Some(i) = token.find(' ') {
    token = &token[i + 1..]
  }

  let ClaimConstructor {
    user_id,
    username,
    reg_state,
    exp: _
  } = Keys::new().verify_claim(&token).unwrap();

  let res = match state.authn
      .finish_passkey_registration(&reg, &reg_state) {
    Ok(sk) => {
      let mut users_guard = state.users.lock().await;
      users_guard.keys
          .entry(user_id)
          .and_modify(|keys| keys.push(sk.clone()))
          .or_insert_with(|| vec![sk.clone()] );

      users_guard.name_to_id.insert(username, user_id);

      StatusCode::OK
    }
    Err(e) => {
      eprintln!("{}", e);
      StatusCode::BAD_REQUEST
      // AxumResponse::builder()
      //   .status(StatusCode::BAD_REQUEST)
      //   .body("".to_string().into())
      //   .unwrap()
    },
  };

  res
}

async fn auth_middleware<B>(
  request: Request<B>,
  next: Next<B>
) -> Result<AxumResponse, StatusCode> {
  let headers = request.headers();
  let token = headers.get(header::AUTHORIZATION);

  match token {
    Some(token) => {
      let mut token = token.to_str().unwrap();

      if let Some(i) = token.find(' ') {
        token = &token[i + 1..];
      }

      let claim = Keys::new().verify_auth(token);

      match claim {
        Ok(_) => {
          Ok(next.run(request).await)
        }
        Err(e) => {
          println!("Token invalid {}", e);
          Err(StatusCode::UNAUTHORIZED)
        }
      }
    }
    None => Err(StatusCode::UNAUTHORIZED)

  }
}

// The ideal thing would be if I could find out some kind of way to reuse the user id extraction
// logic, so that I could plop down a third endpoint.

// I suppose that could be a class or something ?

// POST request
async fn start_authentication(
  state: State<AppState>,
  Json(body): Json<User>
// ) -> response::Result<impl IntoResponse> {
) -> response::Result<impl IntoResponse> {
  let username = body.name;

  let users_guard = state.users.lock().await;

  let user_unique_id = users_guard
      .name_to_id
      .get(&username)
      .clone()
      .ok_or(StatusCode::UNAUTHORIZED);

  let help = match user_unique_id {
    Ok(unique_id) => {
      let credentials =
          users_guard.keys
              .get(&unique_id)
              .ok_or(StatusCode::IM_A_TEAPOT).unwrap();

      let res = match state
          .authn
          .start_passkey_authentication(credentials) {
        Ok((rcr, auth_state)) => {
          let exp = (Utc::now() + Duration::minutes(1)).timestamp();
          let claim = AuthConstructor {
            user_id: unique_id.clone(),
            username,
            auth_state,
            exp: exp as usize
          };
          let token = Keys::new().tokenize_auth(claim);

          Response::response_builder(StatusCode::OK, token)
              .body::<String>(serde_json::to_string(&rcr).unwrap().into())
              .unwrap()

        }
        Err(e) => {
          eprintln!("{}", e);

          AxumResponse::builder()
              .status(StatusCode::UNAUTHORIZED)
              .body("".to_string())
              .unwrap()
        }
      };
      res
    }
    _ => {
      AxumResponse::builder()
          .status(StatusCode::UNAUTHORIZED)
          .body("".to_string())
          .unwrap()
    }
  };
  Ok(help)
}

async fn finish_authentication<'buf>(
  headers: HeaderMap,
  state: State<AppState>,
  Json(auth): Json<PublicKeyCredential>
) -> Result<impl IntoResponse, &'buf str> {

  let mut token = headers
      .get(header::AUTHORIZATION)
      .unwrap()
      .to_str()
      .unwrap();

  if let Some(i) = token.find(' ') {
    token = &token[i + 1..]
  }

  let keys = Keys::new();

  let AuthConstructor {
    user_id,
    username,
    auth_state,
    exp: _,
  } = keys.verify_auth(&token).unwrap();

  let res: AxumResponse<Body> = match state
      .authn
      .finish_passkey_authentication(&auth, &auth_state) {
    Ok(auth_result) => {
      let mut users_guard = state.users.lock().await;

      users_guard.keys
          .get_mut(&user_id)
          .map(|keys|
              keys.iter_mut().for_each(|sk| {
                // let size = std::mem::size_of_val(sk.cred_id());
                sk.update_credential(&auth_result);
              })
          ).ok_or("We goofed").unwrap();

      // Contrary to the JWT token standard, the user can be signed in for MAX
      // 15 minutes.
      let user = LoggedInUser {
        username,
        uuid: user_id,
        exp: (Utc::now() + Duration::minutes(15)).timestamp() as usize,
      };


      // You are logged in, awesome, create a jwt with a token that contains
      // user information, THIS is the token that gets sent back and forth
      // This token will need to validated and NOT just unwrapped.
      let user_token = keys.tokenize_user(user);

      let now = Local::now();
      let fifteen_minutes = Duration::minutes(15);
      let expires = now + fifteen_minutes;
      let formatted_expires
          = expires.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
      let cookie
          = format!("token={}; HttpOnly; SameSite=Strict; Expires={}; Path=/; Secure", user_token,
                    formatted_expires);

      AxumResponse::builder()
          .status(StatusCode::OK)
          .header(
            header::SET_COOKIE,
            HeaderValue::from_str(cookie.as_str()).unwrap())
          .header(header::COOKIE, HeaderValue::from_str(cookie.as_str()).unwrap())
          .header(header::ACCESS_CONTROL_EXPOSE_HEADERS, header::COOKIE)
          .body("".to_string().into())
          .unwrap()
    },
    Err(e) => {
      println!("Not okay challenge {}", e);
      AxumResponse::builder().status(StatusCode::BAD_REQUEST).body("".to_string().into()).unwrap()
    }
  };
  Ok(res)
}


async fn start_password_creation(
  headers: HeaderMap,
  state: State<AppState>,
  Json(body): Json<User>
) -> impl IntoResponse {
  // Instead of sending the username in a body, we should use the cookie header...
  let username = body.name;

  let users_guard = state.users.lock().await;

  let user_unique_id = users_guard
      .name_to_id
      .get(&username)
      .clone()
      .ok_or(());

  if user_unique_id.is_err() {
    return AxumResponse::builder().status(StatusCode::UNAUTHORIZED).body("".to_string()).unwrap();
  }

  let unique_id = user_unique_id.unwrap();

  let credentials =
    users_guard.keys
      .get(&unique_id)
      .ok_or(StatusCode::IM_A_TEAPOT).unwrap();

  let res = match state
    .authn
    .start_passkey_authentication(credentials) {
    Ok((rcr, auth_state)) => {

      let exp = (Utc::now() + Duration::minutes(1)).timestamp();

      let claim = AuthConstructor {
        user_id: unique_id.clone(),
        username,
        auth_state,
        exp: exp as usize,
      };

      let token = Keys::new().tokenize_auth(claim);

      Response::response_builder(StatusCode::OK, token)
          .body::<String>(serde_json::to_string(&rcr).unwrap().into())
          .unwrap()
    }
    _ => {
      axum::http::Response::builder()
          .status(StatusCode::UNAUTHORIZED)
          .body("".to_string())
          .unwrap()
    }
  };
  res
}

async fn end_password_creation<'buf>(
  headers: HeaderMap,
  state: State<AppState>,
  Json(auth): Json<ValidationOfPassword>,
) -> Result<impl IntoResponse, &'buf str> { // signature should be impl IntoResponse

  let mut token = headers.get(header::COOKIE).unwrap().to_str().unwrap();

  if let Some(i) = token.find('=') {
    token = &token[i + 1..];
  }

  let verified = Keys::new().verify_user(&token);

  // TEMP until middleware is in place
  if verified.is_err() {
    // Change signature, add return here
    axum::http::Response::builder()
      .status(StatusCode::UNAUTHORIZED)
      .body("".to_string())
      .unwrap();
  }

  let mut token = headers
      .get(header::AUTHORIZATION)
      .unwrap()
      .to_str()
      .unwrap();

  if let Some(i) = token.find(' ') {
    token = &token[i + 1..]
  }

  let keys = Keys::new();

  let verified_auth_state = keys.verify_auth(&token);

  if verified_auth_state.is_err() {
    // Change signature, add return here.
    axum::http::Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .body("".to_string())
        .unwrap();
  }

  let unwrapped = verified_auth_state.unwrap();
  let auth_state = unwrapped.auth_state;
  let user_id = unwrapped.user_id;

  let credentials = auth.credentials;
  let user_data = auth.user_data;

  // let id_as_vec: Vec<u8> = credentials.id.chars().map(|x| x as u8).collect();
  let password = &user_data.password;

  if password.len() == 0 {
    // Change signature, add return here
    axum::http::Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body("".to_string())
        .unwrap();
  }

  let LoggedInUser {
    username,
    uuid: _,
    exp: _,
  } = verified.unwrap();

  let res: AxumResponse<Body> = match state.
      authn
      .finish_passkey_authentication(&credentials, &auth_state) {
    Ok(auth_result) => {
      let mut users_guard = state.users.lock().await;

      let id_as_vec = auth_result.cred_id().0.to_vec();
      encrypt_and_store(username, &password, user_data.website, id_as_vec).await;

      // AT SOME POINT, THIS WILL BE GOTTEN FORM THE DATABASE INSTEAD OF IN-
      // MEMORY
      users_guard.keys
          .get_mut(&user_id)
          .map(|keys|
              keys.iter_mut().for_each(|sk| {
                sk.update_credential(&auth_result);
              })
          ).ok_or("We goofed").unwrap();

      AxumResponse::builder()
          .status(StatusCode::OK)
          .body("".to_string().into())
          .unwrap()
    },
    Err(_) => {
      AxumResponse::builder()
          .status(StatusCode::UNAUTHORIZED)
          .body("".to_string().into())
          .unwrap()
    },
  };

  Ok(res)
}

