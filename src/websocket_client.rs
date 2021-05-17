use crate::model::{Config, ConfigSample, Measurement, MetricType, Sample};

use chrono::Utc;
use regex::Regex;
use serde::Deserialize;
use serde_xml_rs::from_str;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use uuid::Uuid;

use websocket::client::ClientBuilder;
use websocket::OwnedMessage;

#[derive(Debug)]
pub struct WebsocketClientConfig {
    host_address: String,
    host_port: u32,
    login_code: String,
}

impl WebsocketClientConfig {
    pub fn new(
        host_address: String,
        host_port: u32,
        login_code: String,
    ) -> Result<Self, Box<dyn Error>> {
        let config = Self {
            host_address,
            host_port,
            login_code,
        };

        println!("{:?}", config);

        Ok(config)
    }

    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        let host_address = env::var("WEBSOCKET_HOST_IP").unwrap_or("127.0.0.1".to_string());
        let host_port: u32 = env::var("WEBSOCKET_HOST_PORT")
            .unwrap_or("8214".to_string())
            .parse()?;
        let login_code = env::var("WEBSOCKET_LOGIN_CODE")?;

        Self::new(host_address, host_port, login_code)
    }
}

pub struct WebsocketClient {
    config: WebsocketClientConfig,
}

impl WebsocketClient {
    pub fn new(config: WebsocketClientConfig) -> Self {
        Self { config }
    }

    pub fn get_measurement(
        &self,
        config: Config,
        last_measurement: Option<Measurement>,
    ) -> Result<Measurement, Box<dyn Error>> {
        println!("Reading measurement from alpha innotec heatpump...");

        let mut measurement = Measurement {
            id: Uuid::new_v4().to_string(),
            source: String::from("jarvis-alpha-innotec-exporter"),
            location: config.location.clone(),
            samples: Vec::new(),
            measured_at_time: Utc::now(),
        };

        let connection = ClientBuilder::new(&format!(
            "ws://{}:{}",
            self.config.host_address, self.config.host_port
        ))?
        .origin(format!("http://{}", self.config.host_address))
        .add_protocol("Lux_WS")
        .connect_insecure()?;

        let (mut receiver, mut sender) = connection.split()?;

        // login
        let navigation = self.login(&mut receiver, &mut sender)?;

        // get measurement samples
        let grouped_sample_configs =
            self.group_sample_configs_per_navigation(config.sample_configs);

        measurement.samples = self.get_samples(
            grouped_sample_configs,
            &mut receiver,
            &mut sender,
            navigation,
        )?;

        match last_measurement {
            Some(lm) => {
                measurement.samples = self.sanitize_samples(measurement.samples, lm.samples)
            }
            None => {}
        }

        println!("Read measurement from alpha innotec heatpump");

        Ok(measurement)
    }

    fn group_sample_configs_per_navigation(
        &self,
        sample_configs: Vec<ConfigSample>,
    ) -> HashMap<String, Vec<ConfigSample>> {
        let mut grouped_sample_configs: HashMap<String, Vec<ConfigSample>> = HashMap::new();

        //   for _, sc := range sampleConfigs {
        //     if _, ok := groupedSampleConfigs[sc.Navigation]; !ok {
        //       groupedSampleConfigs[sc.Navigation] = []apiv1.ConfigSample{}
        //     }
        //     groupedSampleConfigs[sc.Navigation] = append(groupedSampleConfigs[sc.Navigation], sc)
        //   }

        //   return

        grouped_sample_configs
    }

    fn send_and_await(
        &self,
        receiver: &mut websocket::receiver::Reader<std::net::TcpStream>,
        sender: &mut websocket::sender::Writer<std::net::TcpStream>,
        message: websocket::OwnedMessage,
    ) -> Result<String, Box<dyn Error>> {
        let _ = sender.send_message(&message)?;

        for message in receiver.incoming_messages() {
            match message? {
                OwnedMessage::Text(text) => {
                    return Ok(text);
                }
                OwnedMessage::Close(_) => {
                    // return a close
                    sender.send_message(&OwnedMessage::Close(None))?;
                }
                OwnedMessage::Ping(data) => {
                    // return a pong
                    sender.send_message(&OwnedMessage::Pong(data))?;
                }
                OwnedMessage::Pong(_) => {}
                OwnedMessage::Binary(_) => {}
            }
        }

        Err(Box::<dyn Error>::from(
            "No response received for login message",
        ))
    }

    fn login(
        &self,
        receiver: &mut websocket::receiver::Reader<std::net::TcpStream>,
        sender: &mut websocket::sender::Writer<std::net::TcpStream>,
    ) -> Result<Navigation, Box<dyn Error>> {
        let response_message = self.send_and_await(
            receiver,
            sender,
            websocket::OwnedMessage::Text(format!("LOGIN;{}", self.config.login_code)),
        )?;

        let navigation = self.get_navigation_from_response(response_message)?;

        Ok(navigation)
    }

    fn get_samples(
        &self,
        grouped_sample_configs: HashMap<String, Vec<ConfigSample>>,
        receiver: &mut websocket::receiver::Reader<std::net::TcpStream>,
        sender: &mut websocket::sender::Writer<std::net::TcpStream>,

        navigation: Navigation,
    ) -> Result<Vec<Sample>, Box<dyn Error>> {
        let mut samples = Vec::new();

        for (nav, sample_configs) in grouped_sample_configs {
            println!("Fetching values from page {}...", nav);
            let navigation_id = navigation.get_navigation_item_id(&nav)?;
            let response_message = self.send_and_await(
                receiver,
                sender,
                websocket::OwnedMessage::Text(format!("GET;{}", navigation_id)),
            )?;

            println!(
                "Reading {} values from response for page {}...",
                sample_configs.len(),
                nav
            );
            for sample_config in sample_configs.iter() {
                let value = self.get_item_from_response(&sample_config.item, &response_message)?;

                samples.push(Sample {
                    entity_type: sample_config.entity_type,
                    entity_name: sample_config.entity_name.clone(),
                    sample_type: sample_config.sample_type,
                    sample_name: sample_config.sample_name.clone(),
                    metric_type: sample_config.metric_type,
                    value: value * sample_config.value_multiplier,
                });
            }
        }

        Ok(samples)
    }

    fn get_navigation_from_response(
        &self,
        response_message: String,
    ) -> Result<Navigation, Box<dyn Error>> {
        let navigation: Navigation = from_str(&response_message)?;

        Ok(navigation)
    }

    fn get_item_from_response(
        &self,
        item: &String,
        response_message: &String,
    ) -> Result<f64, Box<dyn Error>> {
        // <Content><item id='0x4816ac'><name>Aanvoer</name><value>22.0°C</value></item><item id='0x44fdcc'><name>Retour</name><value>22.0°C</value></item><item id='0x4807dc'><name>Retour berekend</name><value>23.0°C</value></item><item id='0x45e1bc'><name>Heetgas</name><value>38.0°C</value></item><item id='0x448894'><name>Buitentemperatuur</name><value>11.6°C</value></item><item id='0x48047c'><name>Gemiddelde temp.</name><value>13.1°C</value></item><item id='0x457724'><name>Tapwater gemeten</name><value>54.2°C</value></item><item id='0x45e97c'><name>Tapwater ingesteld</name><value>57.0°C</value></item><item id='0x45a41c'><name>Bron-in</name><value>10.5°C</value></item><item id='0x480204'><name>Bron-uit</name><value>10.3°C</value></item><item id='0x4803cc'><name>Menggroep2-aanvoer</name><value>22.0°C</value></item><item id='0x4609cc'><name>Menggr2-aanv.ingest.</name><value>19.0°C</value></item><item id='0x45a514'><name>Zonnecollector</name><value>5.0°C</value></item><item id='0x461ecc'><name>Zonneboiler</name><value>150.0°C</value></item><item id='0x4817a4'><name>Externe energiebron</name><value>5.0°C</value></item><item id='0x4646b4'><name>Aanvoer max.</name><value>66.0°C</value></item><item id='0x45e76c'><name>Zuiggasleiding comp.</name><value>19.4°C</value></item><item id='0x4607d4'><name>Comp. verwarming</name><value>37.7°C</value></item><item id='0x43e60c'><name>Oververhitting</name><value>4.8 K</value></item><name>Temperaturen</name></Content>

        let re = Regex::new(&format!(
            r"<item id='[^']*'><name>{}<\/name><value>(-?[0-9.]+|---)[^<]*<\/value><\/item>",
            item
        ))?;
        let matches = match re.captures(&response_message) {
            Some(m) => m,
            None => {
                return Err(Box::<dyn Error>::from(format!(
                    "No match for item {}",
                    item
                )));
            }
        };

        if matches.len() != 2 {
            return Err(Box::<dyn Error>::from(format!(
                "No match for item {}",
                item
            )));
        }

        return match matches.get(1) {
            None => Ok(0.0),
            Some(m) => {
                let value = m.as_str();
                if value == "---" {
                    return Ok(0.0);
                }
                Ok(value.parse()?)
            }
        };
    }

    fn sanitize_samples(
        &self,
        current_samples: Vec<Sample>,
        last_samples: Vec<Sample>,
    ) -> Vec<Sample> {
        let mut sanitized_samples: Vec<Sample> = Vec::new();

        for current_sample in current_samples.into_iter() {
            // check if there's a corresponding sample in lastSamples and see if the difference with it's value isn't too large
            let mut sanitize = false;
            for last_sample in last_samples.iter() {
                if current_sample.entity_type == last_sample.entity_type
                    && current_sample.entity_name == last_sample.entity_name
                    && current_sample.sample_type == last_sample.sample_type
                    && current_sample.sample_name == last_sample.sample_name
                    && current_sample.metric_type == last_sample.metric_type
                {
                    if current_sample.metric_type == MetricType::Counter
                        && current_sample.value / last_sample.value > 1.1
                    {
                        sanitize = true;
                        println!("Value for {} is more than 10 percent larger than the last sampled value {}, keeping previous value instead", current_sample.sample_name, last_sample.value);
                        sanitized_samples.push(last_sample.clone());
                    }

                    break;
                }
            }

            if !sanitize {
                sanitized_samples.push(current_sample);
            }
        }

        sanitized_samples
    }
}

#[derive(Debug, Deserialize)]
struct Navigation {
    id: String,                 // `xml:"id,attr"`
    items: Vec<NavigationItem>, // `xml:"item"`
}

#[derive(Debug, Deserialize)]
#[serde(rename = "item")]
struct NavigationItem {
    id: String,                 //           `xml:"id,attr"`
    name: String,               //           `xml:"name"`
    items: Vec<NavigationItem>, // `xml:"item"`
}

impl Navigation {
    fn get_navigation_item_id(&self, item_path: &String) -> Result<String, Box<dyn Error>> {
        // itemPathParts := strings.Split(itemPath, " > ")
        // items := n.Items
        // for _, p := range itemPathParts {
        //   exists := false
        //   for _, i := range items {
        //     if p == i.Name {
        //       exists = true
        //       navigationID = i.ID
        //       items = i.Items
        //       break
        //     }
        //   }

        //   if !exists {
        //     return navigationID, fmt.Errorf("Item %v does not exist", p)
        //   }
        // }

        // return
        Ok("".to_string())
    }
}

// func TestMarshal(t *testing.T) {
// 	t.Run("ReturnsXmlString", func(t *testing.T) {

// 		navigation := getNavigation()

// 		// act
// 		xmlString, err := xml.Marshal(navigation)

// 		assert.Nil(t, err)
// 		assert.Equal(t, "<Navigation id=\"0x45cd88\"><item id=\"0x45df90\"><name>Informatie</name><item id=\"0x45df90\"><name>Temperaturen</name></item><item id=\"0x455968\"><name>Ingangen</name></item></item><item id=\"0x450798\"><name>Instelling</name></item><item id=\"0x3dc420\"><name>Klokprogramma</name></item><item id=\"0x45c7b0\"><name>Toegang: Gebruiker</name></item></Navigation>", string(xmlString))
// 	})
// }

// func TestUnmarshal(t *testing.T) {
// 	t.Run("ReturnsXmlString", func(t *testing.T) {

// 		var navigation Navigation
// 		xmlString := "<Navigation id=\"0x45cd88\"><item id=\"0x45df90\"><name>Informatie</name><item id=\"0x45df90\"><name>Temperaturen</name></item><item id=\"0x455968\"><name>Ingangen</name></item></item><item id=\"0x450798\"><name>Instelling</name></item><item id=\"0x3dc420\"><name>Klokprogramma</name></item><item id=\"0x45c7b0\"><name>Toegang: Gebruiker</name></item></Navigation>"

// 		// act
// 		err := xml.Unmarshal([]byte(xmlString), &navigation)

// 		assert.Nil(t, err)
// 		assert.Equal(t, 4, len(navigation.Items))
// 		assert.Equal(t, "Informatie", navigation.Items[0].Name)
// 		assert.Equal(t, 2, len(navigation.Items[0].Items))
// 		assert.Equal(t, "Temperaturen", navigation.Items[0].Items[0].Name)
// 		assert.Equal(t, "0x45df90", navigation.Items[0].Items[0].ID)
// 		assert.Equal(t, "Ingangen", navigation.Items[0].Items[1].Name)
// 		assert.Equal(t, "0x455968", navigation.Items[0].Items[1].ID)
// 	})
// }

// func TestGetNavigationItemID(t *testing.T) {
// 	t.Run("ReturnsItemAtTopLevel", func(t *testing.T) {

// 		navigation := getNavigation()

// 		// act
// 		itemID, err := navigation.GetNavigationItemID("Informatie")

// 		assert.Nil(t, err)
// 		assert.Equal(t, "0x45df90", itemID)
// 	})

// 	t.Run("ReturnsItemNestedInsideTopLevelItem", func(t *testing.T) { // <Navigation id='0x45cd88'><item id='0x45e068'><name>Informatie</name><item id='0x45df90'><name>Temperaturen</name></item><item id='0x455968'><name>Ingangen</name></item><item id='0x455760'><name>Uitgangen</name></item><item id='0x45bf10'><name>Aflooptijden</name></item><item id='0x456f08'><name>Bedrijfsuren</name></item><item id='0x4643a8'><name>Storingsbuffer</name></item><item id='0x3ddfa8'><name>Afschakelingen</name></item><item id='0x45d840'><name>Installatiestatus</name></item><item id='0x460cb8'><name>Energie</name></item><item id='0x4586a8'><name>GBS</name></item></item><item id='0x450798'><name>Instelling</name><item id='0x460bd0'><name>Bedrijfsmode</name></item><item id='0x461170'><name>Temperaturen</name></item><item id='0x462988'><name>Systeeminstelling</name></item></item><item id='0x3dc420'><name>Klokprogramma</name><readOnly>true</readOnly><item id='0x453560'><name>Verwarmen</name><readOnly>true</readOnly><item id='0x45e118'><name>Week</name></item><item id='0x45df00'><name>5+2</name></item><item id='0x45c200'><name>Dagen (Ma, Di,...)</name></item></item><item id='0x43e8e8'><name>Warmwater</name><readOnly>true</readOnly><item id='0x4642a8'><name>Week</name></item><item id='0x463940'><name>5+2</name></item><item id='0x463b68'><name>Dagen (Ma, Di,...)</name></item></item><item id='0x3dcc00'><name>Zwembad</name><readOnly>true</readOnly><item id='0x455580'><name>Week</name></item><item id='0x463f78'><name>5+2</name></item><item id='0x462690'><name>Dagen (Ma, Di,...)</name></item></item></item><item id='0x45c7b0'><name>Toegang: Gebruiker</name></item></Navigation>

// 		navigation := getNavigation()

// 		// act
// 		itemID, err := navigation.GetNavigationItemID("Informatie > Ingangen")

// 		assert.Nil(t, err)
// 		assert.Equal(t, "0x455968", itemID)
// 	})
// }

// func getNavigation() Navigation {

// 	// <Navigation id='0x45cd88'><item id='0x45e068'><name>Informatie</name><item id='0x45df90'><name>Temperaturen</name></item><item id='0x455968'><name>Ingangen</name></item><item id='0x455760'><name>Uitgangen</name></item><item id='0x45bf10'><name>Aflooptijden</name></item><item id='0x456f08'><name>Bedrijfsuren</name></item><item id='0x4643a8'><name>Storingsbuffer</name></item><item id='0x3ddfa8'><name>Afschakelingen</name></item><item id='0x45d840'><name>Installatiestatus</name></item><item id='0x460cb8'><name>Energie</name></item><item id='0x4586a8'><name>GBS</name></item></item><item id='0x450798'><name>Instelling</name><item id='0x460bd0'><name>Bedrijfsmode</name></item><item id='0x461170'><name>Temperaturen</name></item><item id='0x462988'><name>Systeeminstelling</name></item></item><item id='0x3dc420'><name>Klokprogramma</name><readOnly>true</readOnly><item id='0x453560'><name>Verwarmen</name><readOnly>true</readOnly><item id='0x45e118'><name>Week</name></item><item id='0x45df00'><name>5+2</name></item><item id='0x45c200'><name>Dagen (Ma, Di,...)</name></item></item><item id='0x43e8e8'><name>Warmwater</name><readOnly>true</readOnly><item id='0x4642a8'><name>Week</name></item><item id='0x463940'><name>5+2</name></item><item id='0x463b68'><name>Dagen (Ma, Di,...)</name></item></item><item id='0x3dcc00'><name>Zwembad</name><readOnly>true</readOnly><item id='0x455580'><name>Week</name></item><item id='0x463f78'><name>5+2</name></item><item id='0x462690'><name>Dagen (Ma, Di,...)</name></item></item></item><item id='0x45c7b0'><name>Toegang: Gebruiker</name></item></Navigation>

// 	return Navigation{
// 		ID: "0x45cd88",
// 		Items: []NavigationItem{
// 			{
// 				ID:   "0x45df90",
// 				Name: "Informatie",
// 				Items: []NavigationItem{
// 					{
// 						ID:   "0x45df90",
// 						Name: "Temperaturen",
// 					},
// 					{
// 						ID:   "0x455968",
// 						Name: "Ingangen",
// 					},
// 				},
// 			},
// 			{
// 				ID:   "0x450798",
// 				Name: "Instelling",
// 			},
// 			{
// 				ID:   "0x3dc420",
// 				Name: "Klokprogramma",
// 			},
// 			{
// 				ID:   "0x45c7b0",
// 				Name: "Toegang: Gebruiker",
// 			},
// 		},
// 	}

// }

// func TestGetMeasurement(t *testing.T) {
// 	t.Run("ReturnsMeasurement", func(t *testing.T) {

// 		if testing.Short() {
// 			t.Skip("skipping test in short mode.")
// 		}

// 		client, err := NewClient("192.168.178.94", 8214, "999999")
// 		assert.Nil(t, err)

// 		config := apiv1.Config{
// 			Location: "My address",
// 		}

// 		// act
// 		measurement, err := client.GetMeasurement(config, nil)

// 		assert.Nil(t, err)
// 		assert.Equal(t, "My address", measurement.Location)
// 	})

// 	t.Run("ReturnsMeasurementWithSample", func(t *testing.T) {

// 		if testing.Short() {
// 			t.Skip("skipping test in short mode.")
// 		}

// 		client, err := NewClient("192.168.178.94", 8214, "999999")
// 		assert.Nil(t, err)

// 		config := apiv1.Config{
// 			Location: "My address",
// 			SampleConfigs: []apiv1.ConfigSample{
// 				{
// 					EntityType:      "ENTITY_TYPE_DEVICE",
// 					EntityName:      "Alpha Innotec SWCV 92K3",
// 					SampleType:      "SAMPLE_TYPE_TEMPERATURE",
// 					SampleName:      "Aanvoer",
// 					MetricType:      "METRIC_TYPE_GAUGE",
// 					ValueMultiplier: 1,
// 					Navigation:      "Informatie > Temperaturen",
// 					Item:            "Aanvoer",
// 				},
// 			},
// 		}

// 		// act
// 		measurement, err := client.GetMeasurement(config, nil)

// 		assert.Nil(t, err)
// 		assert.Equal(t, 1, len(measurement.Samples))
// 		assert.Equal(t, "Alpha Innotec SWCV 92K3", measurement.Samples[0].EntityName)
// 		assert.Equal(t, "Aanvoer", measurement.Samples[0].SampleName)
// 		assert.Equal(t, contractsv1.MetricType_METRIC_TYPE_GAUGE, measurement.Samples[0].MetricType)
// 	})
// }

// func TestGetItemFromResponse(t *testing.T) {
// 	t.Run("ReturnsValueForItemWithoutUnit", func(t *testing.T) {

// 		client := client{}

// 		response := `<Content><item id='0x4816ac'><name>Aanvoer</name><value>22.3°C</value></item><item id='0x44fdcc'><name>Retour</name><value>22.0°C</value></item><item id='0x4807dc'><name>Retour berekend</name><value>23.0°C</value></item><item id='0x45e1bc'><name>Heetgas</name><value>38.0°C</value></item><item id='0x448894'><name>Buitentemperatuur</name><value>11.6°C</value></item><item id='0x48047c'><name>Gemiddelde temp.</name><value>13.1°C</value></item><item id='0x457724'><name>Tapwater gemeten</name><value>54.2°C</value></item><item id='0x45e97c'><name>Tapwater ingesteld</name><value>57.0°C</value></item><item id='0x45a41c'><name>Bron-in</name><value>10.5°C</value></item><item id='0x480204'><name>Bron-uit</name><value>10.3°C</value></item><item id='0x4803cc'><name>Menggroep2-aanvoer</name><value>22.0°C</value></item><item id='0x4609cc'><name>Menggr2-aanv.ingest.</name><value>19.0°C</value></item><item id='0x45a514'><name>Zonnecollector</name><value>5.0°C</value></item><item id='0x461ecc'><name>Zonneboiler</name><value>150.0°C</value></item><item id='0x4817a4'><name>Externe energiebron</name><value>5.0°C</value></item><item id='0x4646b4'><name>Aanvoer max.</name><value>66.0°C</value></item><item id='0x45e76c'><name>Zuiggasleiding comp.</name><value>19.4°C</value></item><item id='0x4607d4'><name>Comp. verwarming</name><value>37.7°C</value></item><item id='0x43e60c'><name>Oververhitting</name><value>4.8 K</value></item><name>Temperaturen</name></Content>`

// 		// act
// 		value, err := client.getItemFromResponse("Aanvoer", []byte(response))

// 		assert.Nil(t, err)
// 		assert.Equal(t, float64(22.3), value)
// 	})

// 	t.Run("ReturnsErrorIfItemIdIsNotInResponse", func(t *testing.T) {

// 		client := client{}

// 		response := `<Content><item id='0x4816ac'><name>Aanvoer</name><value>22.3°C</value></item><item id='0x44fdcc'><name>Retour</name><value>22.0°C</value></item><item id='0x4807dc'><name>Retour berekend</name><value>23.0°C</value></item><item id='0x45e1bc'><name>Heetgas</name><value>38.0°C</value></item><item id='0x448894'><name>Buitentemperatuur</name><value>11.6°C</value></item><item id='0x48047c'><name>Gemiddelde temp.</name><value>13.1°C</value></item><item id='0x457724'><name>Tapwater gemeten</name><value>54.2°C</value></item><item id='0x45e97c'><name>Tapwater ingesteld</name><value>57.0°C</value></item><item id='0x45a41c'><name>Bron-in</name><value>10.5°C</value></item><item id='0x480204'><name>Bron-uit</name><value>10.3°C</value></item><item id='0x4803cc'><name>Menggroep2-aanvoer</name><value>22.0°C</value></item><item id='0x4609cc'><name>Menggr2-aanv.ingest.</name><value>19.0°C</value></item><item id='0x45a514'><name>Zonnecollector</name><value>5.0°C</value></item><item id='0x461ecc'><name>Zonneboiler</name><value>150.0°C</value></item><item id='0x4817a4'><name>Externe energiebron</name><value>5.0°C</value></item><item id='0x4646b4'><name>Aanvoer max.</name><value>66.0°C</value></item><item id='0x45e76c'><name>Zuiggasleiding comp.</name><value>19.4°C</value></item><item id='0x4607d4'><name>Comp. verwarming</name><value>37.7°C</value></item><item id='0x43e60c'><name>Oververhitting</name><value>4.8 K</value></item><name>Temperaturen</name></Content>`

// 		// act
// 		_, err := client.getItemFromResponse("BestaatNiet", []byte(response))

// 		assert.NotNil(t, err)
// 	})

// 	t.Run("ReturnsValueForItemWithoutUnitForPressure", func(t *testing.T) {

// 		client := client{}

// 		response := `<Content><item id='0x4e7944'><name>ASD</name><value>Aan</value></item><item id='0x4ffbfc'><name>EVU</name><value>Aan</value></item><item id='0x4ef3b4'><name>HD</name><value>Uit</value></item><item id='0x4dac64'><name>MOT</name><value>Aan</value></item><item id='0x4ca4c4'><name>SWT</name><value>Uit</value></item><item id='0x4fa864'><name>Analoog-In 21</name><value>0.00 V</value></item><item id='0x4d5f1c'><name>Analoog-In 22</name><value>0.00 V</value></item><item id='0x4e6a3c'><name>HD</name><value>8.10 bar</value></item><item id='0x4ca47c'><name>ND</name><value>8.38 bar</value></item><item id='0x4e8004'><name>Debiet</name><value>1200 l/h</value></item><name>Ingangen</name></Content>`

// 		// act
// 		value, err := client.getItemFromResponse("HD", []byte(response))

// 		assert.Nil(t, err)
// 		assert.Equal(t, float64(8.10), value)
// 	})
// }

// func TestGetNavigationFromResponse(t *testing.T) {
// 	t.Run("ReturnsValueForItemWithoutUnit", func(t *testing.T) {

// 		client := client{}

// 		response := `<Navigation id='0x45cd88'><item id='0x45e068'><name>Informatie</name><item id='0x45df90'><name>Temperaturen</name></item><item id='0x455968'><name>Ingangen</name></item><item id='0x455760'><name>Uitgangen</name></item><item id='0x45bf10'><name>Aflooptijden</name></item><item id='0x456f08'><name>Bedrijfsuren</name></item><item id='0x4643a8'><name>Storingsbuffer</name></item><item id='0x3ddfa8'><name>Afschakelingen</name></item><item id='0x45d840'><name>Installatiestatus</name></item><item id='0x460cb8'><name>Energie</name></item><item id='0x4586a8'><name>GBS</name></item></item><item id='0x450798'><name>Instelling</name><item id='0x460bd0'><name>Bedrijfsmode</name></item><item id='0x461170'><name>Temperaturen</name></item><item id='0x462988'><name>Systeeminstelling</name></item></item><item id='0x3dc420'><name>Klokprogramma</name><readOnly>true</readOnly><item id='0x453560'><name>Verwarmen</name><readOnly>true</readOnly><item id='0x45e118'><name>Week</name></item><item id='0x45df00'><name>5+2</name></item><item id='0x45c200'><name>Dagen (Ma, Di,...)</name></item></item><item id='0x43e8e8'><name>Warmwater</name><readOnly>true</readOnly><item id='0x4642a8'><name>Week</name></item><item id='0x463940'><name>5+2</name></item><item id='0x463b68'><name>Dagen (Ma, Di,...)</name></item></item><item id='0x3dcc00'><name>Zwembad</name><readOnly>true</readOnly><item id='0x455580'><name>Week</name></item><item id='0x463f78'><name>5+2</name></item><item id='0x462690'><name>Dagen (Ma, Di,...)</name></item></item></item><item id='0x45c7b0'><name>Toegang: Gebruiker</name></item></Navigation>`

// 		// act
// 		navigation, err := client.getNavigationFromResponse([]byte(response))

// 		assert.Nil(t, err)
// 		assert.Equal(t, 4, len(navigation.Items))
// 		assert.Equal(t, "Informatie", navigation.Items[0].Name)
// 		assert.Equal(t, 10, len(navigation.Items[0].Items))
// 		assert.Equal(t, "Temperaturen", navigation.Items[0].Items[0].Name)
// 		assert.Equal(t, "0x45df90", navigation.Items[0].Items[0].ID)
// 	})
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_() {
        let xml_string = "<Navigation id=\"0x45cd88\"><item id=\"0x45df90\"><name>Informatie</name><item id=\"0x45df90\"><name>Temperaturen</name></item><item id=\"0x455968\"><name>Ingangen</name></item></item><item id=\"0x450798\"><name>Instelling</name></item><item id=\"0x3dc420\"><name>Klokprogramma</name></item><item id=\"0x45c7b0\"><name>Toegang: Gebruiker</name></item></Navigation>";

        // act
        let navigation: Navigation = from_str(xml_string).unwrap();

        assert_eq!(navigation.items.len(), 4);
        assert_eq!(navigation.items[0].name, "Informatie".to_string());
        assert_eq!(navigation.items[0].items.len(), 1);
        assert_eq!(
            navigation.items[0].items[0].name,
            "Temperaturen".to_string()
        );
        assert_eq!(navigation.items[0].items[0].id, "0x45df90".to_string());
        assert_eq!(navigation.items[0].items[1].name, "Ingangen".to_string());
        assert_eq!(navigation.items[0].items[1].id, "0x455968".to_string());
    }
}
