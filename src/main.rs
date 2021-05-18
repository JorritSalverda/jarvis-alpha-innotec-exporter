mod bigquery_client;
mod config_client;
mod exporter_service;
mod model;
mod state_client;
mod websocket_client;

use bigquery_client::{BigqueryClient, BigqueryClientConfig};
use config_client::{ConfigClient, ConfigClientConfig};
use exporter_service::{ExporterService, ExporterServiceConfig};
use state_client::{StateClient, StateClientConfig};
use websocket_client::{WebsocketClient, WebsocketClientConfig};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let websocket_client_config = WebsocketClientConfig::from_env()?;
    let websocket_client = WebsocketClient::new(websocket_client_config);

    let state_client_config = StateClientConfig::from_env().await?;
    let state_client = StateClient::new(state_client_config);

    let bigquery_client_config = BigqueryClientConfig::from_env().await?;
    let bigquery_client = BigqueryClient::new(bigquery_client_config);

    let config_client_config = ConfigClientConfig::from_env()?;
    let config_client = ConfigClient::new(config_client_config);

    let exporter_service_config = ExporterServiceConfig::new(
        config_client,
        bigquery_client,
        state_client,
        websocket_client,
    )?;
    let exporter_service = ExporterService::new(exporter_service_config);

    exporter_service.run().await?;

    Ok(())
}
