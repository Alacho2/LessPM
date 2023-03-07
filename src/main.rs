use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use warp::Filter;



#[tokio::main]
async fn main() {
  let hello = warp::path!("hello" / String)
    .map(|name| format!("Hello, {}", name));

  let addr = SocketAddr::new(
    IpAddr::V4(Ipv4Addr::new(127,0,0,1)),8080
  );


  warp::serve(hello).run(addr).await;
}
