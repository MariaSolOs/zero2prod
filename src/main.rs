use std::io::{self, Error};
use zero2prod::{configuration, startup::Application, telemetry};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let subscriber = telemetry::get_subscriber("zero2prod", "info", io::stdout);
    telemetry::init_subscriber(subscriber);

    let config = configuration::get_configuration().expect("Failed to read configuration.");

    let app = Application::build(config).await?;
    app.run_until_stopped().await?;

    Ok(())
}
