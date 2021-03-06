package websocket

import (
	"encoding/xml"
	"errors"
	"fmt"
	"io/ioutil"
	"net/http"
	"net/url"
	"os"
	"os/signal"
	"regexp"
	"strconv"
	"sync"
	"time"

	apiv1 "github.com/JorritSalverda/jarvis-alpha-innotec-exporter/api/v1"
	contractsv1 "github.com/JorritSalverda/jarvis-contracts-golang/contracts/v1"
	"github.com/google/uuid"
	"github.com/gorilla/websocket"
	gwebsocket "github.com/gorilla/websocket"
	"github.com/rs/zerolog/log"
)

// Client is the interface for connecting to a websocket device via ethernet
type Client interface {
	GetMeasurement(config apiv1.Config, lastMeasurement *contractsv1.Measurement) (measurement contractsv1.Measurement, err error)
}

// NewClient returns new websocket.Client
func NewClient(host string, port int, loginCode string) (Client, error) {
	if host == "" {
		return nil, fmt.Errorf("Please set the ip address of your websocket device on your local network")
	}
	if port <= 0 {
		return nil, fmt.Errorf("Please set the websocket port of your websocket device on your local network to its default 8214")
	}

	return &client{
		host:      host,
		port:      port,
		loginCode: loginCode,
	}, nil
}

type client struct {
	host      string
	port      int
	loginCode string

	interrupt       chan os.Signal
	done            chan struct{}
	commandChannel  chan string
	responseChannel chan []byte

	awaitingResponse bool
	teardown         bool
}

func (c *client) GetMeasurement(config apiv1.Config, lastMeasurement *contractsv1.Measurement) (measurement contractsv1.Measurement, err error) {

	u := url.URL{Scheme: "ws", Host: fmt.Sprintf("%v:%v", c.host, c.port), Path: "/"}

	log.Info().Msgf("Dialing %v://%v%v...", u.Scheme, u.Host, u.Path)

	requestHeader := http.Header{
		"Origin":                 []string{fmt.Sprintf("http://%v", u.Host)},
		"Sec-WebSocket-Protocol": []string{"Lux_WS"},
	}

	connection, resp, err := gwebsocket.DefaultDialer.Dial(u.String(), requestHeader)
	if err != nil {
		if err == gwebsocket.ErrBadHandshake {
			if resp.Body != nil {
				defer resp.Body.Close()
				body, err := ioutil.ReadAll(resp.Body)
				log.Debug().Str("body", string(body)).Err(err).Msgf("handshake failed body")
			}

			log.Warn().Interface("response", resp.Body).Msgf("handshake failed with status %d", resp.StatusCode)
		}
		return
	}
	defer connection.Close()

	// set up handlers for sending commands and receiving responses
	c.interrupt = make(chan os.Signal, 1)
	signal.Notify(c.interrupt, os.Interrupt)

	c.done = make(chan struct{})
	waitGroup := &sync.WaitGroup{}

	c.responseChannel = make(chan []byte)
	go func() {
		waitGroup.Add(1)
		defer waitGroup.Done()

		if err := c.receiveResponse(connection); err != nil {
			log.Error().Err(err).Msg("Failure receiving responses")
		}
	}()

	c.commandChannel = make(chan string)
	go func() {
		waitGroup.Add(1)
		defer waitGroup.Done()

		if err := c.sendCommands(connection); err != nil {
			log.Error().Err(err).Msg("Failure sending commands")
		}
	}()

	measurement = contractsv1.Measurement{
		ID:             uuid.New().String(),
		Source:         "jarvis-alpha-innotec-exporter",
		Location:       config.Location,
		Samples:        []*contractsv1.Sample{},
		MeasuredAtTime: time.Now().UTC(),
	}

	// login
	navigation, err := c.login()
	if err != nil {
		return
	}

	groupedSampleConfigs := c.groupSampleConfigsPerNavigation(config.SampleConfigs)
	measurement.Samples, err = c.getSamples(config, groupedSampleConfigs, connection, navigation)

	if lastMeasurement != nil {
		measurement.Samples = c.sanitizeSamples(measurement.Samples, lastMeasurement.Samples)
	}

	log.Info().Msgf("Done issueing commands, stopping send/receive handlers...")
	c.teardown = true
	close(c.interrupt)
	waitGroup.Wait()

	return
}

func (c *client) getSamples(config apiv1.Config, groupedSampleConfigs map[string][]apiv1.ConfigSample, connection *gwebsocket.Conn, navigation Navigation) (samples []*contractsv1.Sample, err error) {

	samples = []*contractsv1.Sample{}

	for nav, sampleConfigs := range groupedSampleConfigs {

		log.Info().Msgf("Fetching values from page %v...", nav)

		// get id for navigation
		navigationID, e := navigation.GetNavigationItemID(nav)
		if e != nil {
			return samples, e
		}

		// get response for navigation item
		response, e := c.sendAndAwait(fmt.Sprintf("GET;%v", navigationID))
		if e != nil {
			return samples, fmt.Errorf("Failed navigating to %v: %w", navigationID, e)
		}

		log.Info().Msgf("Reading %v values from response for page %v...", len(sampleConfigs), nav)

		// get all requested values from navigation response
		for _, sc := range sampleConfigs {
			value, e := c.getItemFromResponse(sc.Item, response)
			if e != nil {
				return samples, e
			}

			// init sample from config
			sample := contractsv1.Sample{
				EntityType: sc.EntityType,
				EntityName: sc.EntityName,
				SampleType: sc.SampleType,
				SampleName: sc.SampleName,
				MetricType: sc.MetricType,
			}

			// convert sample to float and correct
			sample.Value = value * sc.ValueMultiplier

			samples = append(samples, &sample)
		}
	}

	return
}

func (c *client) groupSampleConfigsPerNavigation(sampleConfigs []apiv1.ConfigSample) (groupedSampleConfigs map[string][]apiv1.ConfigSample) {

	groupedSampleConfigs = map[string][]apiv1.ConfigSample{}

	for _, sc := range sampleConfigs {
		if _, ok := groupedSampleConfigs[sc.Navigation]; !ok {
			groupedSampleConfigs[sc.Navigation] = []apiv1.ConfigSample{}
		}
		groupedSampleConfigs[sc.Navigation] = append(groupedSampleConfigs[sc.Navigation], sc)
	}

	return
}

func (c *client) receiveResponse(connection *gwebsocket.Conn) (err error) {
	defer close(c.done)
	for {
		var message []byte
		_, message, err = connection.ReadMessage()
		if c.teardown {
			log.Info().Msg("Completing teardown of serial port listener")
			return nil
		}

		if err != nil {
			if errors.Is(err, gwebsocket.ErrCloseSent) {
				log.Debug().Msg("Connection close is sent")
				return nil
			}
			log.Warn().Err(err).Msg("read error")
			return
		}
		log.Debug().Msgf("read: %s", message)
		if c.awaitingResponse {
			c.responseChannel <- message
		}
	}
}

func (c *client) sendCommands(connection *gwebsocket.Conn) (err error) {

	ticker := time.NewTicker(time.Second)
	defer ticker.Stop()

	for {
		select {
		case command := <-c.commandChannel:
			err = connection.WriteMessage(gwebsocket.TextMessage, []byte(command))
			if err != nil {

				log.Warn().Err(err).Msg("write error")
				return
			}
			log.Debug().Msgf("write: %s", command)

		case <-c.done:
			log.Info().Msg("done")
			return

		case <-c.interrupt:
			log.Info().Msg("interrupt")

			// Cleanly close the connection by sending a close message and then
			// waiting (with timeout) for the server to close the connection.
			err = connection.WriteMessage(gwebsocket.CloseMessage, websocket.FormatCloseMessage(websocket.CloseNormalClosure, ""))
			if err != nil {
				log.Warn().Err(err).Msg("write close error")
				return
			}
			select {
			case <-c.done:
			case <-time.After(time.Second):
			}
			return

		case t := <-ticker.C:
			err = connection.WriteMessage(gwebsocket.TextMessage, []byte(t.String()))
			if err != nil {
				log.Warn().Err(err).Msg("write error")
				return
			}
		}
	}
}

func (c *client) sendAndAwait(command string) (response []byte, err error) {

	c.awaitingResponse = true
	defer func() { c.awaitingResponse = false }()

	// issue command
	log.Info().Msgf("Issueing command: %v", command)
	c.commandChannel <- command

	// await response
	select {
	case response = <-c.responseChannel:
		log.Info().Msgf("Received response: %s", response)
		return
	case <-c.interrupt:
		return
	}
}

func (c *client) login() (navigation Navigation, err error) {
	response, err := c.sendAndAwait(fmt.Sprintf("LOGIN;%v", c.loginCode))
	if err != nil {
		return navigation, fmt.Errorf("Failed logging in: %w", err)
	}

	navigation, err = c.getNavigationFromResponse(response)
	if err != nil {
		return
	}

	return
}

func (c *client) getNavigationFromResponse(response []byte) (navigation Navigation, err error) {

	err = xml.Unmarshal(response, &navigation)
	if err != nil {
		return
	}

	return
}

func (c *client) getItemFromResponse(item string, response []byte) (value float64, err error) {

	// <Content><item id='0x4816ac'><name>Aanvoer</name><value>22.0°C</value></item><item id='0x44fdcc'><name>Retour</name><value>22.0°C</value></item><item id='0x4807dc'><name>Retour berekend</name><value>23.0°C</value></item><item id='0x45e1bc'><name>Heetgas</name><value>38.0°C</value></item><item id='0x448894'><name>Buitentemperatuur</name><value>11.6°C</value></item><item id='0x48047c'><name>Gemiddelde temp.</name><value>13.1°C</value></item><item id='0x457724'><name>Tapwater gemeten</name><value>54.2°C</value></item><item id='0x45e97c'><name>Tapwater ingesteld</name><value>57.0°C</value></item><item id='0x45a41c'><name>Bron-in</name><value>10.5°C</value></item><item id='0x480204'><name>Bron-uit</name><value>10.3°C</value></item><item id='0x4803cc'><name>Menggroep2-aanvoer</name><value>22.0°C</value></item><item id='0x4609cc'><name>Menggr2-aanv.ingest.</name><value>19.0°C</value></item><item id='0x45a514'><name>Zonnecollector</name><value>5.0°C</value></item><item id='0x461ecc'><name>Zonneboiler</name><value>150.0°C</value></item><item id='0x4817a4'><name>Externe energiebron</name><value>5.0°C</value></item><item id='0x4646b4'><name>Aanvoer max.</name><value>66.0°C</value></item><item id='0x45e76c'><name>Zuiggasleiding comp.</name><value>19.4°C</value></item><item id='0x4607d4'><name>Comp. verwarming</name><value>37.7°C</value></item><item id='0x43e60c'><name>Oververhitting</name><value>4.8 K</value></item><name>Temperaturen</name></Content>

	pattern := fmt.Sprintf(`<item id='[^']*'><name>%v<\/name><value>(-?[0-9.]+|---)[^<]*<\/value><\/item>`, item)

	re, err := regexp.Compile(pattern)
	if err != nil {
		return
	}

	matches := re.FindStringSubmatch(string(response))
	if err != nil {
		return
	}

	if len(matches) != 2 {
		return value, fmt.Errorf("No match for item %v", item)
	}

	if matches[1] == "---" {
		return value, nil
	}

	value, err = strconv.ParseFloat(matches[1], 64)
	if err != nil {
		return value, fmt.Errorf("Failed parsing float from item %v value %v: %w", item, value, err)
	}

	return
}

func (c *client) sanitizeSamples(currentSamples, lastSamples []*contractsv1.Sample) (sanitizeSamples []*contractsv1.Sample) {

	sanitizeSamples = []*contractsv1.Sample{}
	for _, cs := range currentSamples {
		// check if there's a corresponding sample in lastSamples and see if the difference with it's value isn't too large
		sanitize := false
		for _, ls := range lastSamples {
			if cs.EntityType == ls.EntityType &&
				cs.EntityName == ls.EntityName &&
				cs.SampleType == ls.SampleType &&
				cs.SampleName == ls.SampleName &&
				cs.MetricType == cs.MetricType {
				if cs.MetricType == contractsv1.MetricType_METRIC_TYPE_COUNTER && cs.Value/ls.Value > 1.1 {
					log.Warn().Msgf("Value for %v is more than 10 percent larger than the last sampled value %v, keeping previous value instead", cs, ls.Value)
					sanitizeSamples = append(sanitizeSamples, ls)
				}

				break
			}
		}
		if !sanitize {
			sanitizeSamples = append(sanitizeSamples, cs)
		}
	}

	return
}
