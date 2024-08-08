use anyhow::Result;
use std::{fmt, io};
use tokio::task::JoinError;
use zero2prod::{configuration, issue_delivery_worker, startup::Application, telemetry};

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = telemetry::get_subscriber("zero2prod", "info", io::stdout);
    telemetry::init_subscriber(subscriber);

    let configuration = configuration::get_configuration().expect("Failed to read configuration.");
    let application = Application::build(configuration.clone()).await?;
    let application_task = tokio::spawn(application.run_until_stopped());
    let worker_task = tokio::spawn(issue_delivery_worker::run_worker_until_stopped(
        configuration,
    ));

    tokio::select! {
        o = application_task => report_exit("API", o),
        o = worker_task => report_exit("Background worker", o),
    };

    Ok(())
}

fn report_exit(
    task_name: &str,
    outcome: Result<Result<(), impl fmt::Debug + fmt::Display>, JoinError>,
) {
    match outcome {
        Ok(Ok(())) => tracing::info!("{} has exited.", task_name),
        Ok(Err(e)) => {
            tracing::error!(error.cause_chain = ?e, error.message = %e, "{}' failed.", task_name)
        }
        Err(e) => {
            tracing::error!(error.cause_chain = ?e, error.message = %e, "{}' task failed to complete.", task_name)
        }
    }
}
