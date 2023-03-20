use std::fmt::{Debug};
use std::str::FromStr;
use std::sync::Arc;
use axum::{Router, routing, extract::State, Json, middleware};
use axum::body::{Body};
use axum::http::{
  header,
  HeaderMap,
  HeaderValue,
  Request,
  StatusCode
};
use axum::middleware::{Next};
use axum::response::{IntoResponse, Response as AxumResponse};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Local, Utc};
use mongodb::bson::oid::ObjectId;
use regex::Regex;
use webauthn_rs;
use webauthn_rs::prelude::{PublicKeyCredential, RegisterPublicKeyCredential, Uuid};
use webauthn_rs::Webauthn;
use crate::app_state::AppState;
use crate::db_connection::{DbConnection, RegisteredUser, VaultEntry};
use crate::encryption::{AuthConstructor, ClaimConstructor, EncryptionProcess, Keys, LoggedInUser};
use crate::response::Response;
use crate::user_routes::{process_cookie, process_auth_token, process_claim_token};

async fn test_route(
  // headers: HeaderMap,
  state: State<AppState>,
  // Json(body): Json<Whatever>
) -> StatusCode {
  // dbg!(body);
  /*let cookie_header = headers.get(header::COOKIE);

  let mut logged_in_user = process_cookie(cookie_header);

  if logged_in_user.is_none() {
    return Err(StatusCode::UNAUTHORIZED);
  }


   */
  StatusCode::OK

  /*
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

   */
}

#[derive(Deserialize, Serialize)]
struct ValidationOfPassword {
  credentials: PublicKeyCredential,
  #[serde(rename = "userData")]
  user_data: Option<UserData>,
  #[serde(rename = "objectId")]
  object_id: Option<String>,
  process: Process,
}

#[derive(Debug, Deserialize, Serialize)]
enum Process {
  #[serde(rename = "creation")]
  Creation,
  #[serde(rename = "retrieval")]
  Retrieval,
}

#[derive(Debug, Deserialize, Serialize)]
struct UserData {
  website: String,
  username: String,
  password: String,
}

pub fn api_routes(state: AppState) -> Router {
  Router::new()
    // .route("/test", routing::post(test_route))
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


fn start_passkey_authentication_wrapper(
  authn: &Arc<Webauthn>,
  registered_user: &RegisteredUser,
) -> axum::http::Response<String>{
  let passkey = &[registered_user.passkey.clone()];

  match authn.start_passkey_authentication(passkey) {
    Ok((rcr, auth_state)) => {
      let exp = (Utc::now() + Duration::minutes(1)).timestamp() as usize;
      let claim = AuthConstructor {
        user_id: registered_user.uuid,
        username: registered_user.username.clone(),
        auth_state,
        exp,
      };

      let token = Keys::new().tokenize_auth(claim);

      Response::response_builder(StatusCode::OK, token)
        .body::<String>(serde_json::to_string(&rcr).unwrap().into())
        .unwrap()
    }, Err(_) => {
      axum::http::Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .body("".to_string())
        .unwrap()
    }
  }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct User {
  pub name: String,
}

#[derive(Deserialize, Serialize)]
struct LessPmAuthError<'buf> {
  msg: &'buf str,
}

async fn start_registration(
  // headers: HeaderMap,
  state: State<AppState>,
  Json(body): Json<User>
) -> impl IntoResponse {
  let username = body.name;
  let db = DbConnection::new().await;

  let registered = db.get_registered_user(username.to_string()).await;

  // TODO(Håvard): Check token.
  // If this is the case, we should map the same uuid to that new user
  // For now, we just avoid a username hijacking.
  // if registered.is_some() && process_cookie(headers.get(header::COOKIE)) { }
  if registered.is_some() {
    let less_pm_auth_error = LessPmAuthError {
      msg: "Unavailable Username"
    };

    return axum::http::Response::builder()
      .status(StatusCode::UNAUTHORIZED)
      .body(serde_json::to_string(&less_pm_auth_error).unwrap().into())
      .unwrap()
  }

  let user_unique_id = Uuid::new_v4();

  let res = match state
    .authn
    .start_passkey_registration(
    user_unique_id,
    &username,
    &username,
    None // A user can't register more than one device
      // if we ever want to include two devices, this will need to be not none
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

  let auth_claim_token = process_claim_token(token);

  if auth_claim_token.is_none() {
    return Err(StatusCode::UNAUTHORIZED);
  }

  Ok(next.run(request).await)
}

async fn finish_registration(
  header: HeaderMap,
  state: State<AppState>,
  Json(reg): Json<RegisterPublicKeyCredential>
) -> StatusCode {
  let token = header.get(header::AUTHORIZATION);

  let auth_claim_token = process_claim_token(token);

  if auth_claim_token.is_none() {
    return StatusCode::UNAUTHORIZED;
  }

  let ClaimConstructor {
    user_id,
    username,
    reg_state,
    exp: _
  } = auth_claim_token.unwrap();

  let res = match state
    .authn
    .finish_passkey_registration(&reg, &reg_state) {
    Ok(sk) => {
      let db = DbConnection::new().await;

      let user = RegisteredUser {
        username,
        uuid: user_id,
        passkey: sk.clone(),
      };

      db.register_user(user).await;

      StatusCode::OK
    }
    Err(e) => {
      eprintln!("{}", e);
      StatusCode::BAD_REQUEST
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

  let processed_auth_token = process_auth_token(token);

  if processed_auth_token.is_none() {
    return Err(StatusCode::UNAUTHORIZED);
  }

  return Ok(next.run(request).await);

}

// POST request
async fn start_authentication(
  state: State<AppState>,
  Json(body): Json<User>
) -> axum::http::Response<String> {
  let username = body.name;

  let db = DbConnection::new().await;
  let registered_user_opt = db.get_registered_user(username.clone()).await;

  // TODO(Håvard): Change signature
  if registered_user_opt.is_none() {
    let res = axum::http::Response::builder()
      .status(StatusCode::UNAUTHORIZED)
      .body("".to_string())
      .unwrap();
    return res; // counter-intuitive, but that's rust for you
  }

  let register_user = registered_user_opt.unwrap();

  start_passkey_authentication_wrapper(&state.authn, &register_user)
}

async fn finish_authentication<'buf>(
  headers: HeaderMap,
  state: State<AppState>,
  Json(auth): Json<PublicKeyCredential>
) -> axum::http::Response<String> {

  let verified_auth_token
    = process_auth_token(headers.get(header::AUTHORIZATION));

  let err_response = axum::http::Response::builder()
    .status(StatusCode::UNAUTHORIZED)
    .body("".to_string())
    .unwrap();

  if verified_auth_token.is_none() {
    return err_response;

  }

  let AuthConstructor {
    user_id,
    username,
    auth_state,
    exp: _,
  } = verified_auth_token.unwrap();

  return match state
    .authn
    .finish_passkey_authentication(&auth, &auth_state) {
    // Ok(auth_result) => { for the TODO below
    Ok(_) => {
      let db = DbConnection::new().await;
      let registered_user_opt = db.get_registered_user(username.clone()).await;

      if registered_user_opt.is_none() {
        return err_response;
      }

      /*
      // TODO(Håvard): Update the key(s) and put it back. Maybe
      // let mut users_guard = state.users.lock().await;
      // users_guard.keys
      //     .get_mut(&old_user_id)
      //     .map(|keys|
      //         keys.iter_mut().for_each(|sk| {
      //           let size = std::mem::size_of_val(sk.cred_id());
                // sk.update_credential(&auth_result);
              // })
          // ).ok_or("We goofed").unwrap();

      // Contrary to the JWT token standard, the user can be signed in for MAX
      // 15 minutes.
       */
      let user = LoggedInUser {
        username,
        uuid: user_id,
        exp: (Utc::now() + Duration::minutes(15)).timestamp() as usize,
      };

      let user_token = Keys::new().tokenize_user(user);

      let now = Local::now();
      let fifteen_minutes = Duration::minutes(15);
      let expires = now + fifteen_minutes;
      let formatted_expires
        = expires.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
      let cookie
        = format!("token={}; HttpOnly; SameSite=Strict; Expires={}; Path=/; Secure", user_token,
                  formatted_expires);

      axum::http::Response::builder()
        .status(StatusCode::OK)
        .header(
          header::SET_COOKIE,
          HeaderValue::from_str(cookie.as_str()).unwrap())
        .header(header::COOKIE, HeaderValue::from_str(cookie.as_str()).unwrap())
        .body("".to_string())
        .unwrap()
    },
    Err(e) => {
      println!("Not okay challenge {}", e);
      axum::http::Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body("".to_string())
        .unwrap()
    }
  };
}


async fn start_password_creation(
  headers: HeaderMap,
  state: State<AppState>,
) -> axum::http::Response<String> {
  let process_cookie
    = process_cookie(headers.get(header::COOKIE));

  let response_err = axum::http::Response::builder()
    .status(StatusCode::UNAUTHORIZED)
    .body("".to_string())
    .unwrap();

  if process_cookie.is_none() {
    return response_err;
  }

  let LoggedInUser {
    username,
    uuid: _,
    exp: _,
  } = process_cookie.unwrap();

  let db = DbConnection::new().await;

  let user_opt = db.get_registered_user(username).await;

  if user_opt.is_none() {
    return response_err;
  }

  let user_from_db = user_opt.unwrap();

  start_passkey_authentication_wrapper(&state.authn, &user_from_db)
}

// An idea is to expand upon this end point so you can reuse it to
// Adding an extra property to ValidationOfPassword called "process"
// The call functions based on that once things are authenticated.
async fn end_password_creation<'buf>(
  headers: HeaderMap,
  state: State<AppState>,
  Json(auth): Json<ValidationOfPassword>
) -> StatusCode { // I don't want this to return a StatusCode anymore, but an impl IntoResponse

  let verified
    = process_cookie(headers.get(header::COOKIE));
  let verified_auth_state
    = process_auth_token(headers.get(header::AUTHORIZATION));

  if verified.is_none() || verified_auth_state.is_none() {
    return StatusCode::UNAUTHORIZED;
  }

  let auth_state = verified_auth_state.unwrap().auth_state;
  let credentials = auth.credentials;

  return match state.
    authn
    .finish_passkey_authentication(&credentials, &auth_state) {
    Ok(auth_result) => {

      let process = auth.process;
      let id_as_vec = auth_result.cred_id().0.to_vec();

      return match process {
        Process::Creation => password_creation(
          &id_as_vec,
          verified.unwrap(),
          auth.user_data).await,
        Process::Retrieval => retrieve_one_password(&id_as_vec, &auth.object_id).await,
      }
    },
    Err(_) => {
      StatusCode::UNAUTHORIZED
    },
  };
}

fn is_valid_object_id(id: &str) -> bool {
  let re = Regex::new(r"^[0-9a-fA-F]{24}$").unwrap();
  re.is_match(id)
}

async fn retrieve_one_password(validator_vec: &Vec<u8>, id: &Option<String>) -> StatusCode {
  if id.is_none() {
    return StatusCode::BAD_REQUEST
  }

  let string_slice = id.as_ref().unwrap().as_str();

  let valid_key = is_valid_object_id(string_slice);
  let object_id = ObjectId::from_str(string_slice);

  if !valid_key || object_id.is_err() {
    return StatusCode::BAD_REQUEST;
  }

  let db = DbConnection::new().await;
  let value = db.get_one_from_vault(object_id.unwrap()).await;

  if value.is_none() {
    return StatusCode::BAD_REQUEST;
  }

  let value = value.unwrap();
  let whatever = EncryptionProcess {
    salt: value.random_padding,
    nonce: value.nonce,
    key_padding: value.key_padding,
    base64: value.password,
  };
  let process = EncryptionProcess::end(&validator_vec, whatever);
  dbg!(process);
  StatusCode::OK
}

async fn password_creation(
  validator_vec: &Vec<u8>,
  logged_in_user: LoggedInUser,
  user_data: Option<UserData>,
) -> StatusCode {
  if user_data.is_none() {
    return StatusCode::UNAUTHORIZED
  }

  let LoggedInUser { username: _, uuid, exp: _ } = logged_in_user;

  let UserData { website, username: username_to_store, password } = user_data.unwrap();

  if password.len() == 0 || website.len() == 0 {
    return StatusCode::UNAUTHORIZED;
  }

  let EncryptionProcess { salt, nonce, key_padding, base64 }
    = EncryptionProcess::start(&validator_vec, password.as_str());
  
  let uuid_as_str = uuid.to_string();

  let vault_entry = VaultEntry {
    username: username_to_store,
    password: base64,
    website,
    uuid: uuid_as_str,
    nonce,
    key_padding,
    random_padding: salt,
  };

  let db = DbConnection::new().await;
  db.insert_one_to_vault(vault_entry).await.unwrap()
}
