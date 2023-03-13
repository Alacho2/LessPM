use axum::http::{header, Response as AxumResponse, StatusCode};
use axum::http::response::Builder;

pub struct Response {}

impl Response {
  pub fn response_builder(status_code: StatusCode, token: String) -> Builder {
    AxumResponse::builder()
      .status(status_code)
      .header(header::CONTENT_TYPE, "application/json")
      .header(header::AUTHORIZATION, &format!("Bearer {}", token))
      .header(header::ACCESS_CONTROL_EXPOSE_HEADERS, "Authorization")
      .header(header::ACCESS_CONTROL_EXPOSE_HEADERS, header::COOKIE)
      .header(header::ACCESS_CONTROL_EXPOSE_HEADERS, header::SET_COOKIE)
  }
}
