use std::collections::HashMap;
use std::sync::{Arc};
use tokio::sync::Mutex;
use chrono::format::format;
use webauthn_rs::prelude::{Passkey, Url, Uuid, Webauthn, WebauthnBuilder};

pub struct Data {
  pub name_to_id: HashMap<String, Uuid>,
  pub keys: HashMap<Uuid, Vec<Passkey>>,
}

#[derive(Clone)]
pub struct AppState {
  pub authn: Arc<Webauthn>,
  pub users: Arc<Mutex<Data>>,
}

impl AppState {
  pub fn new() -> Self {
    let rp_id = "localhost";
    // let url_to_parse = format!("http://{}:{}", host, port);
    let rp_origin = Url::parse("https://localhost:3000").expect("Invalid url there, buddy");

    let auth_builder =
      WebauthnBuilder::new(rp_id, &rp_origin)
        .expect("Invalid WebAuthnConfig Builder");

    let auth_builder = auth_builder.rp_name("LessPM-Axum");

    let webauthn =
      Arc::new(auth_builder.build().expect("Invalid WebAuthnConfig"));

    let users = Arc::new(Mutex::new(Data {
      name_to_id: HashMap::new(),
      keys: HashMap::new(),
    }));

    Self {
      authn: webauthn,
      users
    }
  }
}
