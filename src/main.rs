use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use axum::response::Html;
use axum::{extract::Path, routing::{get, post}, Router, Json};
use axum::http::StatusCode;
use axum_server::tls_rustls::RustlsConfig;
use u2f::protocol::U2f;
use serde::{Deserialize, Serialize};
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
  env::set_var("RUST_BACKTRACE", "1");
  // For now, we are keeping the client here.
  // I'll move it down to the fido api if it turns out that I only need it
  // there.

  let ports = Ports {
    http: 8080,
    https: 3000,
  };

  let config =
    RustlsConfig::from_pem_file(
      PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("keys").join("cert.pem"),
      PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("keys").join("private.pem")
  ).await.unwrap();

  // let config = RustlsConfig::from_pem()
  let stringed_ip = IP.iter()
    .map(|i| i.to_string())
    .collect::<Vec<_>>()
    .join(".");

  let app_state = AppState::new();


  let app = Router::new()
    .nest("/fido", fido_routes::api_routes(app_state)) // a nest that lives under API
    // .route("/", get(handler).post(post_handler)) // just a cute little getter
    // .route("/todo/:id", get(id)) // dynamic paths
    .fallback(|| async move { StatusCode::NOT_FOUND }) // all other paths
    ;


  println!("Server is listening on: http://{}:{}", stringed_ip, PORT);

  let addr = SocketAddr::from((IP, PORT));

  axum::Server::bind(&addr)
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
