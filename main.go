package main

import (
	"encoding/json"
	"io/ioutil"
	"os"
	"path/filepath"
	"runtime"
	"time"

	"github.com/alecthomas/kingpin"
	foundation "github.com/estafette/estafette-foundation"
	"github.com/rs/zerolog/log"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
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
	heatpumpHostIPAddress = kingpin.Flag("heatpump-host-ip", "Host ip address ofheatpump").Default("127.0.0.1").OverrideDefaultFromEnvar("HEATPUMP_HOST_IP").String()
	heatpumpHostPort      = kingpin.Flag("heatpump-host-port", "Host port of heatpump").Default("502").OverrideDefaultFromEnvar("HEATPUMP_HOST_PORT").Int()
	heatpumpUnitID        = kingpin.Flag("heatpump-unit-id", "ModBus unit id of heatpump").Default("3").OverrideDefaultFromEnvar("HEATPUMP_UNIT_ID").Int()

	bigqueryEnable    = kingpin.Flag("bigquery-enable", "Toggle to enable or disable bigquery integration").Default("true").OverrideDefaultFromEnvar("BQ_ENABLE").Bool()
	bigqueryProjectID = kingpin.Flag("bigquery-project-id", "Google Cloud project id that contains the BigQuery dataset").Envar("BQ_PROJECT_ID").Required().String()
	bigqueryDataset   = kingpin.Flag("bigquery-dataset", "Name of the BigQuery dataset").Envar("BQ_DATASET").Required().String()
	bigqueryTable     = kingpin.Flag("bigquery-table", "Name of the BigQuery table").Envar("BQ_TABLE").Required().String()

	measurementFilePath          = kingpin.Flag("state-file-path", "Path to file with state.").Default("/configs/last-measurement.json").OverrideDefaultFromEnvar("MEASUREMENT_FILE_PATH").String()
	measurementFileConfigMapName = kingpin.Flag("state-file-configmap-name", "Name of the configmap with state file.").Default("alpha-innotec-bigquery-exporter").OverrideDefaultFromEnvar("MEASUREMENT_FILE_CONFIG_MAP_NAME").String()
)

func main() {

	// parse command line parameters
	kingpin.Parse()

	// init log format from envvar ESTAFETTE_LOG_FORMAT
	foundation.InitLoggingFromEnv(foundation.NewApplicationInfo(appgroup, app, version, branch, revision, buildDate))

	// init bigquery client
	bigqueryClient, err := NewBigQueryClient(*bigqueryProjectID, *bigqueryEnable)
	if err != nil {
		log.Fatal().Err(err).Msg("Failed creating bigquery client")
	}

	// init bigquery table if it doesn't exist yet
	initBigqueryTable(bigqueryClient)

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

	// get previous measurement
	measurementMap := readLastMeasurementFromMeasurementFile()

	client, err := NewHeatpumpClient(*heatpumpHostIPAddress, *heatpumpHostPort, *heatpumpUnitID)
	if err != nil {
		log.Fatal().Err(err).Msg("Failed creating heatpump client")
	}

	totalWhOut, err := client.GetTotalWhOut()
	if err != nil {
		log.Fatal().Err(err).Msg("Failed reading totalWhOut")
	}

	name := "Alpha Innotec SWCV 92 K3"
	// if difference with previous measurement is too large ( > 10 kWh ) ignore, the p1 connection probably returned an incorrect reading
	if previousValueAsFloat64, ok := measurementMap[name]; ok && float64(totalWhOut)-previousValueAsFloat64 > 10*1000 {
		log.Fatal().Msgf("Increase for reading '%v' is %v, more than the allowed 10 kWh, skipping the reading", name, float64(totalWhOut)-previousValueAsFloat64)
	}

	measurement := BigQueryMeasurement{
		Readings: []BigQueryHeatpumpReading{
			{
				Name:    name,
				Reading: float64(totalWhOut),
				Unit:    "Wh",
			},
		},
		InsertedAt: time.Now().UTC(),
	}

	err = bigqueryClient.InsertMeasurement(*bigqueryDataset, *bigqueryTable, measurement)
	if err != nil {
		log.Fatal().Err(err).Msg("Failed inserting measurements into bigquery table")
	}

	writeMeasurementToConfigmap(kubeClientset, measurement)

	log.Info().Msgf("Stored %v readings, exiting...", len(measurement.Readings))
}

func initBigqueryTable(bigqueryClient BigQueryClient) {

	log.Debug().Msgf("Checking if table %v.%v.%v exists...", *bigqueryProjectID, *bigqueryDataset, *bigqueryTable)
	tableExist := bigqueryClient.CheckIfTableExists(*bigqueryDataset, *bigqueryTable)
	if !tableExist {
		log.Debug().Msgf("Creating table %v.%v.%v...", *bigqueryProjectID, *bigqueryDataset, *bigqueryTable)
		err := bigqueryClient.CreateTable(*bigqueryDataset, *bigqueryTable, BigQueryMeasurement{}, "inserted_at", true)
		if err != nil {
			log.Fatal().Err(err).Msg("Failed creating bigquery table")
		}
	} else {
		log.Debug().Msgf("Trying to update table %v.%v.%v schema...", *bigqueryProjectID, *bigqueryDataset, *bigqueryTable)
		err := bigqueryClient.UpdateTableSchema(*bigqueryDataset, *bigqueryTable, BigQueryMeasurement{})
		if err != nil {
			log.Fatal().Err(err).Msg("Failed updating bigquery table schema")
		}
	}
}

func readLastMeasurementFromMeasurementFile() (measurementMap map[string]float64) {

	measurementMap = map[string]float64{}

	// check if last measurement file exists in configmap
	var lastMeasurement BigQueryMeasurement
	if _, err := os.Stat(*measurementFilePath); !os.IsNotExist(err) {
		log.Info().Msgf("File %v exists, reading contents...", *measurementFilePath)

		// read state file
		data, err := ioutil.ReadFile(*measurementFilePath)
		if err != nil {
			log.Fatal().Err(err).Msgf("Failed reading file from path %v", *measurementFilePath)
		}

		log.Info().Msgf("Unmarshalling file %v contents...", *measurementFilePath)

		// unmarshal state file
		if err := json.Unmarshal(data, &lastMeasurement); err != nil {
			log.Fatal().Err(err).Interface("data", data).Msg("Failed unmarshalling last measurement file")
		}

		for _, r := range lastMeasurement.Readings {
			measurementMap[r.Name] = r.Reading
		}
	}

	return measurementMap
}

func writeMeasurementToConfigmap(kubeClientset *kubernetes.Clientset, measurement BigQueryMeasurement) {

	// retrieve configmap
	configMap, err := kubeClientset.CoreV1().ConfigMaps(getCurrentNamespace()).Get(*measurementFileConfigMapName, metav1.GetOptions{})
	if err != nil {
		log.Error().Err(err).Msgf("Failed retrieving configmap %v", *measurementFileConfigMapName)
	}

	// marshal state to json
	measurementData, err := json.Marshal(measurement)
	if configMap.Data == nil {
		configMap.Data = make(map[string]string)
	}

	configMap.Data[filepath.Base(*measurementFilePath)] = string(measurementData)

	// update configmap to have measurement available when the application runs the next time and for other applications
	_, err = kubeClientset.CoreV1().ConfigMaps(getCurrentNamespace()).Update(configMap)
	if err != nil {
		log.Fatal().Err(err).Msgf("Failed updating configmap %v", *measurementFileConfigMapName)
	}

	log.Info().Msgf("Stored measurement in configmap %v...", *measurementFileConfigMapName)
}

func getCurrentNamespace() string {
	namespace, err := ioutil.ReadFile("/var/run/secrets/kubernetes.io/serviceaccount/namespace")
	if err != nil {
		log.Fatal().Err(err).Msg("Failed reading namespace")
	}

	return string(namespace)
}
