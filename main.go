package main

import (
	"context"
	"runtime"

	"github.com/JorritSalverda/jarvis-alpha-innotec-exporter/client/bigquery"
	"github.com/JorritSalverda/jarvis-alpha-innotec-exporter/client/config"
	"github.com/JorritSalverda/jarvis-alpha-innotec-exporter/client/websocket"
	"github.com/alecthomas/kingpin"
	foundation "github.com/estafette/estafette-foundation"
	"github.com/rs/zerolog/log"
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
	err = bigqueryClient.InitBigqueryTable(*bigqueryDataset, *bigqueryTable)
	if err != nil {
		log.Fatal().Err(err).Msg("Failed initializing bigquery table")
	}

	// // create kubernetes api client
	// kubeClientConfig, err := rest.InClusterConfig()
	// if err != nil {
	// 	log.Fatal().Err(err)
	// }
	// // creates the clientset
	// kubeClientset, err := kubernetes.NewForConfig(kubeClientConfig)
	// if err != nil {
	// 	log.Fatal().Err(err)
	// }

	// get previous measurement
	// measurementMap := readLastMeasurementFromMeasurementFile()

	websocketClient, err := websocket.NewClient(*websocketHostIPAddress, *websocketHostPort, *websocketLoginCode)
	if err != nil {
		log.Fatal().Err(err).Msg("Failed creating websocket client")
	}

	measurement, err := websocketClient.GetMeasurement(config)
	if err != nil {
		log.Fatal().Err(err).Msg("Failed ")
	}

	err = bigqueryClient.InsertMeasurement(*bigqueryDataset, *bigqueryTable, measurement)
	if err != nil {
		log.Fatal().Err(err).Msg("Failed inserting measurements into bigquery table")
	}

	// writeMeasurementToConfigmap(kubeClientset, measurement)

	log.Info().Msgf("Stored %v samples, exiting...", len(measurement.Samples))
}

// func readLastMeasurementFromMeasurementFile() (measurementMap map[string]float64) {

// 	measurementMap = map[string]float64{}

// 	// check if last measurement file exists in configmap
// 	var lastMeasurement BigQueryMeasurement
// 	if _, err := os.Stat(*measurementFilePath); !os.IsNotExist(err) {
// 		log.Info().Msgf("File %v exists, reading contents...", *measurementFilePath)

// 		// read state file
// 		data, err := ioutil.ReadFile(*measurementFilePath)
// 		if err != nil {
// 			log.Fatal().Err(err).Msgf("Failed reading file from path %v", *measurementFilePath)
// 		}

// 		log.Info().Msgf("Unmarshalling file %v contents...", *measurementFilePath)

// 		// unmarshal state file
// 		if err := json.Unmarshal(data, &lastMeasurement); err != nil {
// 			log.Fatal().Err(err).Interface("data", data).Msg("Failed unmarshalling last measurement file")
// 		}

// 		for _, r := range lastMeasurement.Readings {
// 			measurementMap[r.Name] = r.Reading
// 		}
// 	}

// 	return measurementMap
// }

// func writeMeasurementToConfigmap(kubeClientset *kubernetes.Clientset, measurement BigQueryMeasurement) {

// 	// retrieve configmap
// 	configMap, err := kubeClientset.CoreV1().ConfigMaps(getCurrentNamespace()).Get(*measurementFileConfigMapName, metav1.GetOptions{})
// 	if err != nil {
// 		log.Error().Err(err).Msgf("Failed retrieving configmap %v", *measurementFileConfigMapName)
// 	}

// 	// marshal state to json
// 	measurementData, err := json.Marshal(measurement)
// 	if configMap.Data == nil {
// 		configMap.Data = make(map[string]string)
// 	}

// 	configMap.Data[filepath.Base(*measurementFilePath)] = string(measurementData)

// 	// update configmap to have measurement available when the application runs the next time and for other applications
// 	_, err = kubeClientset.CoreV1().ConfigMaps(getCurrentNamespace()).Update(configMap)
// 	if err != nil {
// 		log.Fatal().Err(err).Msgf("Failed updating configmap %v", *measurementFileConfigMapName)
// 	}

// 	log.Info().Msgf("Stored measurement in configmap %v...", *measurementFileConfigMapName)
// }

// func getCurrentNamespace() string {
// 	namespace, err := ioutil.ReadFile("/var/run/secrets/kubernetes.io/serviceaccount/namespace")
// 	if err != nil {
// 		log.Fatal().Err(err).Msg("Failed reading namespace")
// 	}

// 	return string(namespace)
// }
