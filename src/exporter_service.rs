use std::error::Error;

use crate::bigquery_client::BigqueryClient;
use crate::config_client::ConfigClient;
use crate::state_client::StateClient;
use crate::websocket_client::WebsocketClient;

pub struct ExporterServiceConfig {
    config_client: ConfigClient,
    bigquery_client: BigqueryClient,
    state_client: StateClient,
    websocket_client: WebsocketClient,
}

impl ExporterServiceConfig {
    pub fn new(
        config_client: ConfigClient,
        bigquery_client: BigqueryClient,
        state_client: StateClient,
        websocket_client: WebsocketClient,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            config_client,
            bigquery_client,
            state_client,
            websocket_client,
        })
    }
}

pub struct ExporterService {
    config: ExporterServiceConfig,
}

impl ExporterService {
    pub fn new(config: ExporterServiceConfig) -> Self {
        Self { config }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config = self.config.config_client.read_config_from_file()?;

        self.config.bigquery_client.init_table().await?;

        let last_measurement = self.config.state_client.read_state()?;

        let measurement = self
            .config
            .websocket_client
            .get_measurement(config, last_measurement)?;

        self.config
            .bigquery_client
            .insert_measurement(&measurement)
            .await?;

        self.config.state_client.store_state(&measurement).await?;

        Ok(())
    }
}
