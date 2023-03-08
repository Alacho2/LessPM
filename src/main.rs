use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use axum::response::Html;
use axum::{extract::Path, routing::{get, post}, Router, Json};
use u2f::protocol::U2f;
use serde::{Deserialize, Serialize};

use u2f::protocol;

mod routes;
mod fido_routes;

const IP: [u8; 4] = [127, 0, 0, 1];
const PORT: u16 = 8080;

const APP_STRING: &'static str = "LessPM-WhereDidWeGo";

#[derive(Clone)]
pub struct U2fClient {
  pub u2f: U2f,
}

#[tokio::main]
async fn main() {

  // For now, we are keeping the client here.
  // I'll move it down to the fido api if it turns out that I only need it
  // there.
  let u2f_client = U2fClient {
    u2f: U2f::new(APP_STRING.into()),
  };


  let app = Router::new()
    .nest("/fido", fido_routes::api_routes(u2f_client)) // a nest that lives under API
    // .route("/", get(handler).post(post_handler)) // just a cute little getter
    // .route("/todo/:id", get(id)) // dynamic paths
    ;

  let stringed_ip = IP.iter()
    .map(|i| i.to_string())
    .collect::<Vec<_>>()
    .join(".");

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
