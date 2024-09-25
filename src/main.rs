use libphext::phext;
use axum::{routing::get, Router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Create a simple route handler
    let app = Router::new().route("/", get(handler));

    // Define the address to serve on
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Serving on http://{}", addr);

    // Run the web server with axum_server
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Simple handler function
async fn handler() -> &'static str {
    "Request for Bot #001"
}