package api

import (
	contractsv1 "github.com/JorritSalverda/jarvis-contracts-golang/contracts/v1"
)

type Config struct {
	Location      string         `yaml:"location"`
	SampleConfigs []ConfigSample `yaml:"sampleConfigs"`
}

type ConfigSample struct {
	// default jarvis config for sample
	DeviceName       string                       `yaml:"deviceName"`
	SampleName       string                       `yaml:"sampleName"`
	AggregationLevel contractsv1.AggregationLevel `yaml:"aggregationLevel"`
	MetricType       contractsv1.MetricType       `yaml:"metricType"`
	SampleType       contractsv1.SampleType       `yaml:"sampleType"`
	SampleUnit       contractsv1.SampleUnit       `yaml:"sampleUnit"`

	// alpha innotec specific config for sample
	ValueMultiplier float64 `yaml:"valueMultiplier"`
	Navigation      string  `yaml:"navigation"`
	Item            string  `yaml:"item"`
}
