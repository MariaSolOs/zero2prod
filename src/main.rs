use sqlx::PgPool;
use std::{io::Error, net::TcpListener};
use zero2prod::{configuration, startup};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let configuration = configuration::get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).expect("Failed to bind random port.");

    startup::run(listener, connection_pool)?.await
}
