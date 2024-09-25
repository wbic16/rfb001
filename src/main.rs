use libphext::phext;
use axum::{routing::get, Router, response::Html};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Create a simple route handler
    let app = Router::new()
        .route("/", get(index))
        .route("/api/v1/leaders", get(leaders))
        .route("/api/v1/scan", get(scan))
    ;

    // Define the address to serve on
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Serving on http://{}", addr);

    // Run the web server with axum_server
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index() -> Html<&'static str> {
    Html("<html><head></head><body>
    <ul>
    <li><a href='/api/v1/leaders'>/api/v1/leaders</a></li>
    <li><a href='/api/v1/scan'>/api/v1/scan</a></li>
    </ul></body></html>")
}

async fn leaders() -> &'static str {
    "@wbic16"
}

async fn scan() -> &'static str {
    "TBD"
}