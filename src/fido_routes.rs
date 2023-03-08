use std::fmt::{Debug, Formatter};
use axum::{Router, routing, extract::State};
// use warp::{Filter, path, post, Rejection, Reply};
// use warp::http::StatusCode;
// use warp::reply::with_status;
use u2f::protocol::{Challenge, U2f};
use crate::{U2fClient};

mod fido;

// pub fn register_request() -> impl Filter<
//   Extract = (impl Reply,),
//   Error = Rejection> + Clone
// {
//   path("register_request_test")
//     .and(post())
//     .and_then(|| async move {
//       let response = with_status("", StatusCode::OK);
//       Ok::<_, Rejection>(response)
//     })
// }
//
// pub fn register_request_u2f() -> impl Filter<
//   Extract = (impl Reply,),
//   Error = Rejection
// > + Clone {
//   path("register_request")
//     .and(post())
//     .and_then(fido::register_request)
// }


/*
fn routes<S>(state: AppState) -> Router<S> {
    Router::new()
        .route("/", get(|_: State<AppState>| async {}))
        .with_state(state)
}

let routes = Router::new().nest("/api", routes(AppState {}));

axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
    .serve(routes.into_make_service())
    .await;
 */


pub fn api_routes(state: U2fClient) -> Router {
  Router::new()
    .route("/register_request", routing::get(register_request))
    .route("/sign_request", routing::get(sign_request))
    .route("/sign_response", routing::post(register_request))
    .route("/register_response", routing::post(register_response))
    .with_state(state)
}

async fn register_request(state: State<U2fClient>) -> String {

  let challenge: Challenge = state.u2f.generate_challenge().unwrap();
  let something = serde_json::to_string(&challenge).unwrap();
  println!("{}", something);

  String::new()
}

async fn sign_request(state: State<U2fClient>) {

}

async fn sign_response(state: State<U2fClient>) {

}

async fn register_response(state: State<U2fClient>) {

}
