use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use axum::response::{Html, IntoResponse, Response as AxumResponse};
use axum::{extract::Path, routing::{get, post}, Router, Json};
use axum::body::Body;
use axum::http::{header, HeaderValue, Method, StatusCode};
use axum_server::tls_rustls::RustlsConfig;
use u2f::protocol::U2f;
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use crate::app_state::AppState;
use crate::response::Response;

mod response;
mod routes;
mod app_state;
mod fido_routes;
mod encryption;
mod user_routes;
mod noncesequencehelper;
mod password;

const IP: [u8; 4] = [127, 0, 0, 1];
const PORT: u16 = 8080;

const APP_STRING: &'static str = "LessPM-WhereDidWeGo";

#[derive(Clone, Copy)]
struct Ports {
  http: u16,
  https: u16,
}

#[tokio::main]
async fn main() {
  // Set variables
  // env::set_var("RUST_BACKTRACE", "1");

  let ports = Ports {
    http: 8080,
    https: 3000,
  };

  let config =
    RustlsConfig::from_pem_file(
      PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("keys").join("certificate.pem"),
      PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("keys").join("privatekey.pem")
  ).await.unwrap();

  let stringed_ip = IP.iter()
    .map(|i| i.to_string())
    .collect::<Vec<String>>()
    .join(".");

  let app_state = AppState::new();

  let app = Router::new()
    .nest("/user", user_routes::user_routes())
    // .route("/auth-me", get(auth_me))
    .nest("/fido", fido_routes::api_routes(app_state)) // a nest that lives under API
    .layer(CorsLayer::new()
      .allow_origin([
        // "chrome-extension://jnpfkofnigkaocfcdcdppaokjkmhjcio".parse::<HeaderValue>().unwrap(),
        "https://localhost:3000".parse::<HeaderValue>().unwrap(),
        "https://localhost:1234".parse::<HeaderValue>().unwrap()
      ])
      .allow_credentials(true)
      .allow_methods([Method::GET, Method::POST])
      .allow_headers(vec![
        header::CONTENT_TYPE,
        header::AUTHORIZATION,
        header::COOKIE,
    ])
    )
    // .route("/", get(handler).post(post_handler)) // just a cute little getter
    // .route("/todo/:id", get(id)) // dynamic paths
    .fallback(|| async move { StatusCode::NOT_FOUND }) // all other paths
    ;


  println!("Server is listening on: http://{}:{}", stringed_ip, PORT);

  let addr = SocketAddr::from((IP, ports.https));

  axum_server::bind_rustls(addr, config)
  // axum::Server::bind(&addr)
    .serve(app.into_make_service())
    .await
    .unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
struct Kake {
  kake: String,
}

// async fn auth_me() -> Html<&'static str> {
//   Html("<script>alert(2)</script>")
// }

async fn post_handler(Json(body): Json<Kake>) {
  dbg!(body);
}

// async fn handler() -> impl IntoResponse {
//   let resp: AxumResponse<Body> = axum::http::Response::builder()
//     .status(StatusCode::OK)
//     .header(header::CONTENT_TYPE, "application/json")
//     .header(header::AUTHORIZATION, &format!("Bearer {}", token))
    // .header(header::ACCESS_CONTROL_EXPOSE_HEADERS, "Authorization")
    // .body("".to_string().into())
    // .unwrap();
  // resp
// }

async fn id(Path(id): Path<String>) {
  dbg!(id);
}
