use std::net::TcpListener;
use sqlx::{PgPool};
use zero2prod::{configuration::get_configuration, startup::run};
use env_logger::Env;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)
        .expect("Failed to bind to port 8000");

    run(listener, connection_pool)?.await
}