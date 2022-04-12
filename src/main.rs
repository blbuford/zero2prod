use std::fmt::{Debug, Display};
use tokio::task::JoinError;
use zero2prod::startup::Application;
use zero2prod::{configuration::get_configuration, telemetry::*};
use zero2prod::issue_delivery_worker::run_worker_until_stopped;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");
    let application = tokio::spawn(
        Application::build(configuration.clone())
        .await?
        .run_until_stopped());

    let worker = tokio::spawn(run_worker_until_stopped(configuration));

    tokio::select! {
        o = application => report_exit("API", o),
        o = worker => report_exit("Background Worker", o),
    };

    Ok(())
}

fn report_exit(
    task_name: &str,
    outcome: Result<Result<(), impl Debug + Display>, JoinError>
) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed",
                task_name
            )
        },
        Err(e) =>{
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed to complete",
                task_name
            )
        }
    }
}