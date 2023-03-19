use std::collections::HashMap;
use std::sync::{Arc};
use tokio::sync::Mutex;

use webauthn_rs::prelude::{Passkey, Url, Uuid, Webauthn, WebauthnBuilder};

#[derive(Clone)]
pub struct AppState {
  pub authn: Arc<Webauthn>,
}

impl AppState {
  pub fn new() -> Self {
    let rp_id = "localhost";
    let rp_origin = Url::parse("https://localhost:1234").expect("Invalid url there, buddy");

    let auth_builder =
      WebauthnBuilder::new(rp_id, &rp_origin)
        .expect("Invalid WebAuthnConfig Builder");

    let auth_builder = auth_builder
      .rp_name("LessPM-Axum");

    let webauthn =
      Arc::new(auth_builder.build().expect("Invalid WebAuthnConfig"));

    Self {
      authn: webauthn,
    }
  }
}
