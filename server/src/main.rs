use std::env;
use std::net::{SocketAddr};
use std::path::PathBuf;
use axum::Router;
use axum::http::{header, HeaderValue, Method, StatusCode};
use axum_server::tls_rustls::RustlsConfig;
use tower_http::cors::{CorsLayer};
use crate::app_state::AppState;
use dotenv::dotenv;

mod response;
mod routes;
mod app_state;
mod fido_routes;
mod encryption;
mod user_routes;
mod noncesequencehelper;
mod password;
mod db_connection;

const IP: [u8; 4] = [127, 0, 0, 1];

#[derive(Clone, Copy)]
struct Ports {
  https: u16,
}

#[tokio::main]
async fn main() {
  dotenv().ok();

  let ports = Ports {
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
    .nest("/fido", fido_routes::api_routes(app_state)) // a nest that lives under API
    .layer(CorsLayer::new()
      .allow_origin([
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
    ).fallback(|| async move { StatusCode::NOT_FOUND });


  println!("Server is listening on: https://{}:{}", stringed_ip, ports.https);

  let addr = SocketAddr::from((IP, ports.https));

  axum_server::bind_rustls(addr, config)
    .serve(app.into_make_service())
    .await
    .unwrap();
}