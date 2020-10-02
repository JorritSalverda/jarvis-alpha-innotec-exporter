package main

import (
	"time"
)

type BigQueryMeasurement struct {
	Readings   []BigQueryHeatpumpReading `bigquery:"readings"`
	InsertedAt time.Time                 `bigquery:"inserted_at"`
}

type BigQueryHeatpumpReading struct {
	Name    string  `bigquery:"name"`
	Reading float64 `bigquery:"reading"`
	Unit    string  `bigquery:"unit"`
}
