package config

import (
	"context"
	"testing"

	contractsv1 "github.com/JorritSalverda/jarvis-contracts-golang/contracts/v1"
	"github.com/stretchr/testify/assert"
)

func TestReadConfigFromFile(t *testing.T) {

	t.Run("ReturnsConfig", func(t *testing.T) {

		ctx := context.Background()
		client, _ := NewClient(ctx)

		// act
		config, err := client.ReadConfigFromFile("./test-config.yaml")

		assert.Nil(t, err)
		assert.Equal(t, "My Home", config.Location)
		assert.Equal(t, 2, len(config.SampleConfigs))
		assert.Equal(t, contractsv1.AggregationLevel_AGGREGATION_LEVEL_DEVICE, config.SampleConfigs[0].AggregationLevel)
		assert.Equal(t, contractsv1.MetricType_METRIC_TYPE_GAUGE, config.SampleConfigs[0].MetricType)
		assert.Equal(t, contractsv1.SampleType_SAMPLE_TYPE_TEMPERATURE, config.SampleConfigs[0].SampleType)
		assert.Equal(t, contractsv1.SampleUnit_SAMPLE_UNIT_DEGREE_CELCIUS, config.SampleConfigs[0].SampleUnit)
	})
}
