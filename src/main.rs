use {
    dotenv::dotenv,
    rust_http_starter::{config::Configuration, log, Result},
    tokio::sync::broadcast,
};

#[tokio::main]
async fn main() -> Result<()> {
    let logger = log::Logger::init().expect("Failed to start logging");

    let (_signal, shutdown) = broadcast::channel(1);
    dotenv().ok();

    let config = Configuration::new().expect("Failed to load config!");
    let result = rust_http_starter::bootstap(shutdown, config).await;

    logger.stop();

    result
}
