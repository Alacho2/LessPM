use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use axum::response::Html;
use axum::{extract::Path, routing::{get, post}, Router, Json};
use axum::http::{header, HeaderValue, Method, StatusCode};
use axum_server::tls_rustls::RustlsConfig;
use u2f::protocol::U2f;
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use crate::app_state::AppState;

mod response;
mod routes;
mod app_state;
mod fido_routes;
mod encryption;

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
    .route("/", get(handler))
    .nest("/fido", fido_routes::api_routes(app_state)) // a nest that lives under API
    .layer(CorsLayer::new()
      .allow_origin([
        "chrome-extension://jnpfkofnigkaocfcdcdppaokjkmhjcio".parse::<HeaderValue>().unwrap(),
        "https://localhost:3000".parse::<HeaderValue>().unwrap()
      ])
      // .allow_origin(Any)
      .allow_credentials(true)
      .allow_methods([Method::GET, Method::POST])
      .allow_headers(vec![
        header::CONTENT_TYPE,
        header::AUTHORIZATION,
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


async fn post_handler(Json(body): Json<Kake>) {
  dbg!(body);
}

async fn handler() -> Html<&'static str> {
  Html("<h1>Hello, World!</h1>")
}

async fn id(Path(id): Path<String>) {
  dbg!(id);
}
