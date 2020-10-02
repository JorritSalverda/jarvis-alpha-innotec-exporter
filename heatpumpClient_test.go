package main

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestNewHeatpumpClient(t *testing.T) {
	t.Run("ReturnsClientForDefaults", func(t *testing.T) {

		// act
		_, err := NewHeatpumpClient("127.0.0.1", 502, 3)

		assert.Nil(t, err)
	})

	t.Run("ReturnsErrorIfHostIsEmpty", func(t *testing.T) {

		// act
		_, err := NewHeatpumpClient("", 502, 3)

		assert.NotNil(t, err)
	})

	t.Run("ReturnsErrorIfPortIsNot502OrBetween49152And65535", func(t *testing.T) {

		// act
		_, err := NewHeatpumpClient("127.0.0.1", 501, 3)

		assert.NotNil(t, err)
	})

	t.Run("ReturnsNoErrorIfPortIsBetween49152And65535", func(t *testing.T) {

		// act
		_, err := NewHeatpumpClient("127.0.0.1", 49152, 3)

		assert.Nil(t, err)
	})
}

func TestGetTotalWhOut(t *testing.T) {
	t.Run("ReadsTotalPVEnergy", func(t *testing.T) {

		if testing.Short() {
			t.Skip("skipping test in short mode.")
		}

		client, err := NewHeatpumpClient("192.168.178.88", 502, 3)
		if !assert.Nil(t, err) {
			return
		}

		totalWhOut, err := client.GetTotalWhOut()
		if !assert.Nil(t, err) {
			return
		}

		assert.True(t, totalWhOut > 13000)
	})
}
