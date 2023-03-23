use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;

use crate::routes::{health_check, subscribe};

/// Starts a Actix Web server and start listening for requests
/// on the given listener configuration.
pub fn run(listener: TcpListener, pool: PgPool) -> Result<Server, std::io::Error> {
    // Wrap the DB connection in web::Data which wraps this instance
    // in an Arc reference that can be cloned.
    let conn_pool = web::Data::new(pool);
    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(conn_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
