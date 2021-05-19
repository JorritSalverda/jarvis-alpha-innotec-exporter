use jarvis_lib::{EntityType, MetricType, SampleType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub location: String,
    pub sample_configs: Vec<ConfigSample>,
}

impl Config {
    pub fn set_defaults(&mut self) {
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
