use mailbolt::startup::run;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // @TODO: Read this from environment
    let port = "8000";
    let listener = TcpListener::bind(format!("127.0.0.1:{}", &port))
        .unwrap_or_else(|_| panic!("Could not bind server to port '{}'", &port));

    println!("Server started on port {}", port);

    run(listener)?.await
}
