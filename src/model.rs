use jarvis_lib::config_client::SetDefaults;
use jarvis_lib::model::{EntityType, MetricType, SampleType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub location: String,
    pub sanitize_samples: bool,
    pub sample_configs: Vec<ConfigSample>,
}

impl SetDefaults for Config {
    fn set_defaults(&mut self) {
        for sample_config in self.sample_configs.iter_mut() {
            sample_config.set_defaults()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConfigSample {
    // default jarvis config for sample
    pub entity_type: EntityType,
    pub entity_name: String,
    pub sample_type: SampleType,
    pub sample_name: String,
    pub metric_type: MetricType,

    // alpha innotec specific config for sample
    pub value_multiplier: f64,
    pub navigation: String,
    pub item: String,
}

impl ConfigSample {
    pub fn set_defaults(&mut self) {
        if self.value_multiplier == 0.0 {
            self.value_multiplier = 1.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jarvis_lib::config_client::{ConfigClient, ConfigClientConfig};
    use jarvis_lib::model::{EntityType, MetricType, SampleType};

    #[test]
    fn read_config_from_file_returns_deserialized_test_file() {
        let config_client =
            ConfigClient::new(ConfigClientConfig::new("test-config.yaml".to_string()).unwrap());

        let config: Config = config_client.read_config_from_file().unwrap();

        assert_eq!(config.location, "My Home".to_string());
        assert_eq!(config.sample_configs.len(), 2);
        assert_eq!(config.sample_configs[0].entity_type, EntityType::Device);
        assert_eq!(
            config.sample_configs[0].entity_name,
            "Alpha Innotec SWCV 92K3".to_string()
        );
        assert_eq!(
            config.sample_configs[0].sample_type,
            SampleType::Temperature
        );
        assert_eq!(config.sample_configs[0].sample_name, "Aanvoer".to_string());
        assert_eq!(config.sample_configs[0].metric_type, MetricType::Gauge);

        assert_eq!(config.sample_configs[0].value_multiplier, 1.0);
        assert_eq!(
            config.sample_configs[0].navigation,
            "Informatie > Temperaturen".to_string()
        );
        assert_eq!(config.sample_configs[0].item, "Aanvoer".to_string());
    }
}
