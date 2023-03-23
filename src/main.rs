use mailbolt::{configuration::get_configuration, startup::run};
use std::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let config = get_configuration().expect("Could not read configuration file");

    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.app_port))
        .unwrap_or_else(|_| panic!("Could not bind server to port '{}'", config.app_port));

    println!("Server started on port {}", config.app_port);

    run(listener)?.await
}
