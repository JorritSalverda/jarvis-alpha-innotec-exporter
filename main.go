package main

import (
	"context"
	"runtime"

	"github.com/JorritSalverda/jarvis-alpha-innotec-exporter/client/bigquery"
	"github.com/JorritSalverda/jarvis-alpha-innotec-exporter/client/config"
	"github.com/JorritSalverda/jarvis-alpha-innotec-exporter/client/state"
	"github.com/JorritSalverda/jarvis-alpha-innotec-exporter/client/websocket"
	"github.com/alecthomas/kingpin"
	foundation "github.com/estafette/estafette-foundation"
	"github.com/rs/zerolog/log"
	"k8s.io/client-go/kubernetes"
	"k8s.io/client-go/rest"
)

var (
	// set when building the application
	appgroup  string
	app       string
	version   string
	branch    string
	revision  string
	buildDate string
	goVersion = runtime.Version()

	// application specific config
	websocketHostIPAddress = kingpin.Flag("websocket-host-ip", "Host ip address ofheatpump").Default("127.0.0.1").OverrideDefaultFromEnvar("WEBSOCKET_HOST_IP").String()
	websocketHostPort      = kingpin.Flag("websocket-host-port", "Host port of heatpump").Default("8214").OverrideDefaultFromEnvar("WEBSOCKET_HOST_PORT").Int()
	websocketLoginCode     = kingpin.Flag("websocket-login-code", "Login code for heatpump").Envar("WEBSOCKET_LOGIN_CODE").Required().String()

	bigqueryEnable    = kingpin.Flag("bigquery-enable", "Toggle to enable or disable bigquery integration").Default("true").OverrideDefaultFromEnvar("BQ_ENABLE").Bool()
	bigqueryInit      = kingpin.Flag("bigquery-init", "Toggle to enable bigquery table initialization").Default("true").OverrideDefaultFromEnvar("BQ_INIT").Bool()
	bigqueryProjectID = kingpin.Flag("bigquery-project-id", "Google Cloud project id that contains the BigQuery dataset").Envar("BQ_PROJECT_ID").Required().String()
	bigqueryDataset   = kingpin.Flag("bigquery-dataset", "Name of the BigQuery dataset").Envar("BQ_DATASET").Required().String()
	bigqueryTable     = kingpin.Flag("bigquery-table", "Name of the BigQuery table").Envar("BQ_TABLE").Required().String()

	configPath                   = kingpin.Flag("config-path", "Path to the config.yaml file").Default("/configs/config.yaml").OverrideDefaultFromEnvar("CONFIG_PATH").String()
	measurementFilePath          = kingpin.Flag("state-file-path", "Path to file with state.").Default("/configs/last-measurement.json").OverrideDefaultFromEnvar("MEASUREMENT_FILE_PATH").String()
	measurementFileConfigMapName = kingpin.Flag("state-file-configmap-name", "Name of the configmap with state file.").Default("jarvis-alpha-innotec-exporter").OverrideDefaultFromEnvar("MEASUREMENT_FILE_CONFIG_MAP_NAME").String()
)

func main() {

	// parse command line parameters
	kingpin.Parse()

	// init log format from envvar ESTAFETTE_LOG_FORMAT
	foundation.InitLoggingFromEnv(foundation.NewApplicationInfo(appgroup, app, version, branch, revision, buildDate))

	// create context to cancel commands on sigterm
	ctx := foundation.InitCancellationContext(context.Background())

	configClient, err := config.NewClient(ctx)
	if err != nil {
		log.Fatal().Err(err).Msg("Failed creating config.Client")
	}

	// read config from yaml file
	config, err := configClient.ReadConfigFromFile(*configPath)
	if err != nil {
		log.Fatal().Err(err).Msgf("Failed loading config from %v", *configPath)
	}

	log.Info().Interface("config", config).Msgf("Loaded config from %v", *configPath)

	// init bigquery client
	bigqueryClient, err := bigquery.NewClient(*bigqueryProjectID, *bigqueryEnable)
	if err != nil {
		log.Fatal().Err(err).Msg("Failed creating bigquery.Client")
	}

	// init bigquery table if it doesn't exist yet
	if *bigqueryInit {
		err = bigqueryClient.InitBigqueryTable(*bigqueryDataset, *bigqueryTable)
		if err != nil {
			log.Fatal().Err(err).Msg("Failed initializing bigquery table")
		}
	}

	// create kubernetes api client
	kubeClientConfig, err := rest.InClusterConfig()
	if err != nil {
		log.Fatal().Err(err)
	}
	// creates the clientset
	kubeClientset, err := kubernetes.NewForConfig(kubeClientConfig)
	if err != nil {
		log.Fatal().Err(err)
	}

	stateClient, err := state.NewClient(ctx, kubeClientset, *measurementFilePath, *measurementFileConfigMapName)
	if err != nil {
		log.Fatal().Err(err).Msg("Failed creating state client")
	}

	websocketClient, err := websocket.NewClient(*websocketHostIPAddress, *websocketHostPort, *websocketLoginCode)
	if err != nil {
		log.Fatal().Err(err).Msg("Failed creating websocket client")
	}

	lastMeasurement, err := stateClient.ReadState(ctx)
	if err != nil {
		log.Fatal().Err(err).Msg("Failed reading last state")
	}

	measurement, err := websocketClient.GetMeasurement(config, lastMeasurement)
	if err != nil {
		log.Fatal().Err(err).Msg("Failed ")
	}

	err = bigqueryClient.InsertMeasurement(*bigqueryDataset, *bigqueryTable, measurement)
	if err != nil {
		log.Fatal().Err(err).Msg("Failed inserting measurements into bigquery table")
	}

	err = stateClient.StoreState(ctx, measurement)
	if err != nil {
		log.Fatal().Err(err).Msg("Failed storing measurements in state file")
	}

	log.Info().Msgf("Stored %v samples, exiting...", len(measurement.Samples))
}
