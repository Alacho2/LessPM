use std::sync::Arc;
use chrono::format::format;
use webauthn_rs::prelude::{Url, Webauthn, WebauthnBuilder};

#[derive(Clone, Debug)]
pub struct AppState {
  pub authn: Arc<Webauthn>,
}

impl AppState {
  pub fn new() -> Self {
    let rp_id = "localhost";
    // let url_to_parse = format!("http://{}:{}", host, port);
    let rp_origin = Url::parse("http://localhost:8080").expect("Invalid url there, buddy");

    let auth_builder =
      WebauthnBuilder::new(rp_id, &rp_origin)
        .expect("Invalid WebAuthnConfig Builder");

    let auth_builder = auth_builder.rp_name("LessPM-Axum");

    let webauthn =
      Arc::new(auth_builder.build().expect("Invalid WebAuthnConfig"));

    Self { authn: webauthn }
  }
}
