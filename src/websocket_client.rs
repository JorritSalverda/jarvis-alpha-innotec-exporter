use crate::model::{Config, Measurement, MetricType, Sample, SampleType};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json;
use std::env;
use std::error::Error;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Debug)]
pub struct WebsocketClientConfig {
    host_address: String,
    host_port: u32,
    login_code: String,
}

impl WebsocketClientConfig {
    pub fn new(host_address: String, host_port: u32, login_code: String) -> Result<Self, Box<dyn Error>> {
        let config = Self { host_address, host_port, login_code };
        
        println!("{:?}", config);

        Ok(config)
    }

    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        let host_address = env::var("WEBSOCKET_HOST_IP").unwrap_or("127.0.0.1".to_string());
        let host_port: u32 = env::var("WEBSOCKET_HOST_PORT").unwrap_or("8214".to_string()).parse()?;
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

        // println!("Discovering devices...");
        // let devices = self.discover_devices()?;

        // for device in devices.iter() {
        //     match &device.system {
        //         Some(system) => {
        //             match &device.e_meter {
        //                 Some(e_meter) => {
        //                     // counter
        //                     measurement.samples.push(Sample {
        //                         entity_type: config.entity_type,
        //                         entity_name: config.entity_name.clone(),
        //                         sample_type: SampleType::ElectricityConsumption,
        //                         sample_name: system.info.alias.clone(),
        //                         metric_type: MetricType::Counter,
        //                         value: e_meter.real_time.total_watt_hour * 3600.0,
        //                     });

        //                     // gauge
        //                     measurement.samples.push(Sample {
        //                         entity_type: config.entity_type,
        //                         entity_name: config.entity_name.clone(),
        //                         sample_type: SampleType::ElectricityConsumption,
        //                         sample_name: system.info.alias.clone(),
        //                         metric_type: MetricType::Gauge,
        //                         value: e_meter.real_time.power_milli_watt / 1000.0,
        //                     });
        //                 }
        //                 None => (),
        //             }
        //         }
        //         None => (),
        //     }
        // }

        match last_measurement {
            Some(lm) => {
                measurement.samples = self.sanitize_samples(measurement.samples, lm.samples)
            }
            None => {}
        }

        println!("Read measurement from alpha innotec heatpump");

        Ok(measurement)
    }








    // func (c *client) GetMeasurement(config apiv1.Config, lastMeasurement *contractsv1.Measurement) (measurement contractsv1.Measurement, err error) {

    //   u := url.URL{Scheme: "ws", Host: fmt.Sprintf("%v:%v", c.host, c.port), Path: "/"}
    
    //   log.Info().Msgf("Dialing %v://%v%v...", u.Scheme, u.Host, u.Path)
    
    //   requestHeader := http.Header{
    //     "Origin":                 []string{fmt.Sprintf("http://%v", u.Host)},
    //     "Sec-WebSocket-Protocol": []string{"Lux_WS"},
    //   }
    
    //   connection, resp, err := gwebsocket.DefaultDialer.Dial(u.String(), requestHeader)
    //   if err != nil {
    //     if err == gwebsocket.ErrBadHandshake {
    //       if resp.Body != nil {
    //         defer resp.Body.Close()
    //         body, err := ioutil.ReadAll(resp.Body)
    //         log.Debug().Str("body", string(body)).Err(err).Msgf("handshake failed body")
    //       }
    
    //       log.Warn().Interface("response", resp.Body).Msgf("handshake failed with status %d", resp.StatusCode)
    //     }
    //     return
    //   }
    //   defer connection.Close()
    
    //   // set up handlers for sending commands and receiving responses
    //   c.interrupt = make(chan os.Signal, 1)
    //   signal.Notify(c.interrupt, os.Interrupt)
    
    //   c.done = make(chan struct{})
    //   waitGroup := &sync.WaitGroup{}
    
    //   c.responseChannel = make(chan []byte)
    //   go func() {
    //     waitGroup.Add(1)
    //     defer waitGroup.Done()
    
    //     if err := c.receiveResponse(connection); err != nil {
    //       log.Error().Err(err).Msg("Failure receiving responses")
    //     }
    //   }()
    
    //   c.commandChannel = make(chan string)
    //   go func() {
    //     waitGroup.Add(1)
    //     defer waitGroup.Done()
    
    //     if err := c.sendCommands(connection); err != nil {
    //       log.Error().Err(err).Msg("Failure sending commands")
    //     }
    //   }()
    
    //   measurement = contractsv1.Measurement{
    //     ID:             uuid.New().String(),
    //     Source:         "jarvis-alpha-innotec-exporter",
    //     Location:       config.Location,
    //     Samples:        []*contractsv1.Sample{},
    //     MeasuredAtTime: time.Now().UTC(),
    //   }
    
    //   // login
    //   navigation, err := c.login()
    //   if err != nil {
    //     return
    //   }
    
    //   groupedSampleConfigs := c.groupSampleConfigsPerNavigation(config.SampleConfigs)
    //   measurement.Samples, err = c.getSamples(config, groupedSampleConfigs, connection, navigation)
    
    //   if lastMeasurement != nil {
    //     measurement.Samples = c.sanitizeSamples(measurement.Samples, lastMeasurement.Samples)
    //   }
    
    //   log.Info().Msgf("Done issueing commands, stopping send/receive handlers...")
    //   c.teardown = true
    //   close(c.interrupt)
    //   waitGroup.Wait()
    
    //   return
    // }
    
    fn get_samples(&self, config: Config, grouped_sample_configs: map[string][]apiv1.ConfigSample, connection: *gwebsocket.Conn, navigation: Navigation) -> Result<Vec<Sample>, Box<dyn Error>> {
      Ok(vec![])
    }

    // func (c *client) getSamples(config apiv1.Config, groupedSampleConfigs map[string][]apiv1.ConfigSample, connection *gwebsocket.Conn, navigation Navigation) (samples []*contractsv1.Sample, err error) {
    
    //   samples = []*contractsv1.Sample{}
    
    //   for nav, sampleConfigs := range groupedSampleConfigs {
    
    //     log.Info().Msgf("Fetching values from page %v...", nav)
    
    //     // get id for navigation
    //     navigationID, e := navigation.GetNavigationItemID(nav)
    //     if e != nil {
    //       return samples, e
    //     }
    
    //     // get response for navigation item
    //     response, e := c.sendAndAwait(fmt.Sprintf("GET;%v", navigationID))
    //     if e != nil {
    //       return samples, fmt.Errorf("Failed navigating to %v: %w", navigationID, e)
    //     }
    
    //     log.Info().Msgf("Reading %v values from response for page %v...", len(sampleConfigs), nav)
    
    //     // get all requested values from navigation response
    //     for _, sc := range sampleConfigs {
    //       value, e := c.getItemFromResponse(sc.Item, response)
    //       if e != nil {
    //         return samples, e
    //       }
    
    //       // init sample from config
    //       sample := contractsv1.Sample{
    //         EntityType: sc.EntityType,
    //         EntityName: sc.EntityName,
    //         SampleType: sc.SampleType,
    //         SampleName: sc.SampleName,
    //         MetricType: sc.MetricType,
    //       }
    
    //       // convert sample to float and correct
    //       sample.Value = value * sc.ValueMultiplier
    
    //       samples = append(samples, &sample)
    //     }
    //   }
    
    //   return
    // }
    
    fn group_sample_configs_per_navigation(&self, sample_configs: Vec<SampleConfig>) -> map[string][]apiv1.ConfigSample {
      Ok(map[string][]apiv1.ConfigSample)
    }

    // func (c *client) groupSampleConfigsPerNavigation(sampleConfigs []apiv1.ConfigSample) (groupedSampleConfigs map[string][]apiv1.ConfigSample) {
    
    //   groupedSampleConfigs = map[string][]apiv1.ConfigSample{}
    
    //   for _, sc := range sampleConfigs {
    //     if _, ok := groupedSampleConfigs[sc.Navigation]; !ok {
    //       groupedSampleConfigs[sc.Navigation] = []apiv1.ConfigSample{}
    //     }
    //     groupedSampleConfigs[sc.Navigation] = append(groupedSampleConfigs[sc.Navigation], sc)
    //   }
    
    //   return
    // }

    fn receive_response(&self, connection *gwebsocket.Conn) -> Result<(), Box<dyn Error>> {
      Ok(())
    }
    
    // func (c *client) receiveResponse(connection *gwebsocket.Conn) (err error) {
    //   defer close(c.done)
    //   for {
    //     var message []byte
    //     _, message, err = connection.ReadMessage()
    //     if c.teardown {
    //       log.Info().Msg("Completing teardown of serial port listener")
    //       return nil
    //     }
    
    //     if err != nil {
    //       if errors.Is(err, gwebsocket.ErrCloseSent) {
    //         log.Debug().Msg("Connection close is sent")
    //         return nil
    //       }
    //       log.Warn().Err(err).Msg("read error")
    //       return
    //     }
    //     log.Debug().Msgf("read: %s", message)
    //     if c.awaitingResponse {
    //       c.responseChannel <- message
    //     }
    //   }
    // }

    fn send_commands(&self, connection: *gwebsocket.Conn) -> Result<(), Box<dyn Error>> {
      Ok(())
    }
    
    // func (c *client) sendCommands(connection *gwebsocket.Conn) (err error) {
    
    //   ticker := time.NewTicker(time.Second)
    //   defer ticker.Stop()
    
    //   for {
    //     select {
    //     case command := <-c.commandChannel:
    //       err = connection.WriteMessage(gwebsocket.TextMessage, []byte(command))
    //       if err != nil {
    
    //         log.Warn().Err(err).Msg("write error")
    //         return
    //       }
    //       log.Debug().Msgf("write: %s", command)
    
    //     case <-c.done:
    //       log.Info().Msg("done")
    //       return
    
    //     case <-c.interrupt:
    //       log.Info().Msg("interrupt")
    
    //       // Cleanly close the connection by sending a close message and then
    //       // waiting (with timeout) for the server to close the connection.
    //       err = connection.WriteMessage(gwebsocket.CloseMessage, websocket.FormatCloseMessage(websocket.CloseNormalClosure, ""))
    //       if err != nil {
    //         log.Warn().Err(err).Msg("write close error")
    //         return
    //       }
    //       select {
    //       case <-c.done:
    //       case <-time.After(time.Second):
    //       }
    //       return
    
    //     case t := <-ticker.C:
    //       err = connection.WriteMessage(gwebsocket.TextMessage, []byte(t.String()))
    //       if err != nil {
    //         log.Warn().Err(err).Msg("write error")
    //         return
    //       }
    //     }
    //   }
    // }


    fn send_and_await(&self, command: String) -> Result<Vec<u8>, Box<dyn Error>> {
      Ok(vec![])
    }
    
    // func (c *client) sendAndAwait(command string) (response []byte, err error) {
    
    //   c.awaitingResponse = true
    //   defer func() { c.awaitingResponse = false }()
    
    //   // issue command
    //   log.Info().Msgf("Issueing command: %v", command)
    //   c.commandChannel <- command
    
    //   // await response
    //   select {
    //   case response = <-c.responseChannel:
    //     log.Info().Msgf("Received response: %s", response)
    //     return
    //   case <-c.interrupt:
    //     return
    //   }
    // }

    fn login(&self) -> Result<Navigation, Box<dyn Error>> {
      Ok(Navigation{})
    }    
    
    // func (c *client) login() (navigation Navigation, err error) {
    //   response, err := c.sendAndAwait(fmt.Sprintf("LOGIN;%v", c.loginCode))
    //   if err != nil {
    //     return navigation, fmt.Errorf("Failed logging in: %w", err)
    //   }
    
    //   navigation, err = c.getNavigationFromResponse(response)
    //   if err != nil {
    //     return
    //   }
    
    //   return
    // }
    
    fn get_navigation_from_response(&self, response: Vec<u8>) -> Result<Navigation, Box<dyn Error>> {
      Ok(Navigation{})
    }    
    

    // func (c *client) getNavigationFromResponse(response []byte) (navigation Navigation, err error) {
    
    //   err = xml.Unmarshal(response, &navigation)
    //   if err != nil {
    //     return
    //   }
    
    //   return
    // }
    
    fn get_item_from_response(&self, item: String, response: Vec<u8>) -> Result<f64, Box<dyn Error>> {
      Ok(0.0)
    }    

    // func (c *client) getItemFromResponse(item string, response []byte) (value float64, err error) {
    
    //   // <Content><item id='0x4816ac'><name>Aanvoer</name><value>22.0°C</value></item><item id='0x44fdcc'><name>Retour</name><value>22.0°C</value></item><item id='0x4807dc'><name>Retour berekend</name><value>23.0°C</value></item><item id='0x45e1bc'><name>Heetgas</name><value>38.0°C</value></item><item id='0x448894'><name>Buitentemperatuur</name><value>11.6°C</value></item><item id='0x48047c'><name>Gemiddelde temp.</name><value>13.1°C</value></item><item id='0x457724'><name>Tapwater gemeten</name><value>54.2°C</value></item><item id='0x45e97c'><name>Tapwater ingesteld</name><value>57.0°C</value></item><item id='0x45a41c'><name>Bron-in</name><value>10.5°C</value></item><item id='0x480204'><name>Bron-uit</name><value>10.3°C</value></item><item id='0x4803cc'><name>Menggroep2-aanvoer</name><value>22.0°C</value></item><item id='0x4609cc'><name>Menggr2-aanv.ingest.</name><value>19.0°C</value></item><item id='0x45a514'><name>Zonnecollector</name><value>5.0°C</value></item><item id='0x461ecc'><name>Zonneboiler</name><value>150.0°C</value></item><item id='0x4817a4'><name>Externe energiebron</name><value>5.0°C</value></item><item id='0x4646b4'><name>Aanvoer max.</name><value>66.0°C</value></item><item id='0x45e76c'><name>Zuiggasleiding comp.</name><value>19.4°C</value></item><item id='0x4607d4'><name>Comp. verwarming</name><value>37.7°C</value></item><item id='0x43e60c'><name>Oververhitting</name><value>4.8 K</value></item><name>Temperaturen</name></Content>
    
    //   pattern := fmt.Sprintf(`<item id='[^']*'><name>%v<\/name><value>(-?[0-9.]+|---)[^<]*<\/value><\/item>`, item)
    
    //   re, err := regexp.Compile(pattern)
    //   if err != nil {
    //     return
    //   }
    
    //   matches := re.FindStringSubmatch(string(response))
    //   if err != nil {
    //     return
    //   }
    
    //   if len(matches) != 2 {
    //     return value, fmt.Errorf("No match for item %v", item)
    //   }
    
    //   if matches[1] == "---" {
    //     return value, nil
    //   }
    
    //   value, err = strconv.ParseFloat(matches[1], 64)
    //   if err != nil {
    //     return value, fmt.Errorf("Failed parsing float from item %v value %v: %w", item, value, err)
    //   }
    
    //   return
    // }

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


// <Navigation id='0x45cd88'><item id='0x45e068'><name>Informatie</name><item id='0x45df90'><name>Temperaturen</name></item><item id='0x455968'><name>Ingangen</name></item><item id='0x455760'><name>Uitgangen</name></item><item id='0x45bf10'><name>Aflooptijden</name></item><item id='0x456f08'><name>Bedrijfsuren</name></item><item id='0x4643a8'><name>Storingsbuffer</name></item><item id='0x3ddfa8'><name>Afschakelingen</name></item><item id='0x45d840'><name>Installatiestatus</name></item><item id='0x460cb8'><name>Energie</name></item><item id='0x4586a8'><name>GBS</name></item></item><item id='0x450798'><name>Instelling</name><item id='0x460bd0'><name>Bedrijfsmode</name></item><item id='0x461170'><name>Temperaturen</name></item><item id='0x462988'><name>Systeeminstelling</name></item></item><item id='0x3dc420'><name>Klokprogramma</name><readOnly>true</readOnly><item id='0x453560'><name>Verwarmen</name><readOnly>true</readOnly><item id='0x45e118'><name>Week</name></item><item id='0x45df00'><name>5+2</name></item><item id='0x45c200'><name>Dagen (Ma, Di,...)</name></item></item><item id='0x43e8e8'><name>Warmwater</name><readOnly>true</readOnly><item id='0x4642a8'><name>Week</name></item><item id='0x463940'><name>5+2</name></item><item id='0x463b68'><name>Dagen (Ma, Di,...)</name></item></item><item id='0x3dcc00'><name>Zwembad</name><readOnly>true</readOnly><item id='0x455580'><name>Week</name></item><item id='0x463f78'><name>5+2</name></item><item id='0x462690'><name>Dagen (Ma, Di,...)</name></item></item></item><item id='0x45c7b0'><name>Toegang: Gebruiker</name></item></Navigation>

struct Navigation  {
	xml_name: String,         // `xml:"Navigation"`
	id:      String,           // `xml:"id,attr"`
	items:   Vec<NavigationItem>, // `xml:"item"`
}

struct NavigationItem {
	xml_name: String, //         `xml:"item"`
	id:      String, //           `xml:"id,attr"`
	name:    String, //           `xml:"name"`
	items:   Vec<NavigationItem>, // `xml:"item"`
}

impl Navigation {
  fn get_navigation_item_id(&self, item_path: String) -> Result<String,Box<dyn Error>> {
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
