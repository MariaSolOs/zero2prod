use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::{
    io::{self, Error},
    net::TcpListener,
};
use zero2prod::{configuration, startup, telemetry};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let subscriber = telemetry::get_subscriber("zero2prod", "info", io::stdout);
    telemetry::init_subscriber(subscriber);

    let config = configuration::get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPool::connect(&config.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");
    let address = format!("127.0.0.1:{}", config.application_port);
    let listener = TcpListener::bind(address).expect("Failed to bind random port.");

    startup::run(listener, connection_pool)?.await
}
