use mailbolt::{
    configuration::get_configuration,
    startup::build,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Init telemetry subscriber to process tracing spans and logs
    let subscriber = get_subscriber("mailbolt".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let config = get_configuration().expect("Could not read configuration YML files");
    let server = build(config).await?;
    server.await?;
    Ok(())
}
