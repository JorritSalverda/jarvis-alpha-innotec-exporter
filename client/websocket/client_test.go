package websocket

import (
	"testing"

	apiv1 "github.com/JorritSalverda/jarvis-alpha-innotec-exporter/api/v1"
	contractsv1 "github.com/JorritSalverda/jarvis-contracts-golang/contracts/v1"
	"github.com/stretchr/testify/assert"
)

func TestGetMeasurement(t *testing.T) {
	t.Run("ReturnsMeasurement", func(t *testing.T) {

		if testing.Short() {
			t.Skip("skipping test in short mode.")
		}

		client, err := NewClient("192.168.178.94", 8214, "999999")
		assert.Nil(t, err)

		config := apiv1.Config{
			Location: "My address",
		}

		// act
		measurement, err := client.GetMeasurement(config, nil)

		assert.Nil(t, err)
		assert.Equal(t, "My address", measurement.Location)
	})

	t.Run("ReturnsMeasurementWithSample", func(t *testing.T) {

		if testing.Short() {
			t.Skip("skipping test in short mode.")
		}

		client, err := NewClient("192.168.178.94", 8214, "999999")
		assert.Nil(t, err)

		config := apiv1.Config{
			Location: "My address",
			SampleConfigs: []apiv1.ConfigSample{
				{
					EntityType:      "ENTITY_TYPE_DEVICE",
					EntityName:      "Alpha Innotec SWCV 92K3",
					SampleType:      "SAMPLE_TYPE_TEMPERATURE",
					SampleName:      "Aanvoer",
					MetricType:      "METRIC_TYPE_GAUGE",
					ValueMultiplier: 1,
					Navigation:      "Informatie > Temperaturen",
					Item:            "Aanvoer",
				},
			},
		}

		// act
		measurement, err := client.GetMeasurement(config, nil)

		assert.Nil(t, err)
		assert.Equal(t, 1, len(measurement.Samples))
		assert.Equal(t, "Alpha Innotec SWCV 92K3", measurement.Samples[0].EntityName)
		assert.Equal(t, "Aanvoer", measurement.Samples[0].SampleName)
		assert.Equal(t, contractsv1.MetricType_METRIC_TYPE_GAUGE, measurement.Samples[0].MetricType)
	})
}

func TestGetItemFromResponse(t *testing.T) {
	t.Run("ReturnsValueForItemWithoutUnit", func(t *testing.T) {

		client := client{}

		response := `<Content><item id='0x4816ac'><name>Aanvoer</name><value>22.3°C</value></item><item id='0x44fdcc'><name>Retour</name><value>22.0°C</value></item><item id='0x4807dc'><name>Retour berekend</name><value>23.0°C</value></item><item id='0x45e1bc'><name>Heetgas</name><value>38.0°C</value></item><item id='0x448894'><name>Buitentemperatuur</name><value>11.6°C</value></item><item id='0x48047c'><name>Gemiddelde temp.</name><value>13.1°C</value></item><item id='0x457724'><name>Tapwater gemeten</name><value>54.2°C</value></item><item id='0x45e97c'><name>Tapwater ingesteld</name><value>57.0°C</value></item><item id='0x45a41c'><name>Bron-in</name><value>10.5°C</value></item><item id='0x480204'><name>Bron-uit</name><value>10.3°C</value></item><item id='0x4803cc'><name>Menggroep2-aanvoer</name><value>22.0°C</value></item><item id='0x4609cc'><name>Menggr2-aanv.ingest.</name><value>19.0°C</value></item><item id='0x45a514'><name>Zonnecollector</name><value>5.0°C</value></item><item id='0x461ecc'><name>Zonneboiler</name><value>150.0°C</value></item><item id='0x4817a4'><name>Externe energiebron</name><value>5.0°C</value></item><item id='0x4646b4'><name>Aanvoer max.</name><value>66.0°C</value></item><item id='0x45e76c'><name>Zuiggasleiding comp.</name><value>19.4°C</value></item><item id='0x4607d4'><name>Comp. verwarming</name><value>37.7°C</value></item><item id='0x43e60c'><name>Oververhitting</name><value>4.8 K</value></item><name>Temperaturen</name></Content>`

		// act
		value, err := client.getItemFromResponse("Aanvoer", []byte(response))

		assert.Nil(t, err)
		assert.Equal(t, float64(22.3), value)
	})

	t.Run("ReturnsErrorIfItemIdIsNotInResponse", func(t *testing.T) {

		client := client{}

		response := `<Content><item id='0x4816ac'><name>Aanvoer</name><value>22.3°C</value></item><item id='0x44fdcc'><name>Retour</name><value>22.0°C</value></item><item id='0x4807dc'><name>Retour berekend</name><value>23.0°C</value></item><item id='0x45e1bc'><name>Heetgas</name><value>38.0°C</value></item><item id='0x448894'><name>Buitentemperatuur</name><value>11.6°C</value></item><item id='0x48047c'><name>Gemiddelde temp.</name><value>13.1°C</value></item><item id='0x457724'><name>Tapwater gemeten</name><value>54.2°C</value></item><item id='0x45e97c'><name>Tapwater ingesteld</name><value>57.0°C</value></item><item id='0x45a41c'><name>Bron-in</name><value>10.5°C</value></item><item id='0x480204'><name>Bron-uit</name><value>10.3°C</value></item><item id='0x4803cc'><name>Menggroep2-aanvoer</name><value>22.0°C</value></item><item id='0x4609cc'><name>Menggr2-aanv.ingest.</name><value>19.0°C</value></item><item id='0x45a514'><name>Zonnecollector</name><value>5.0°C</value></item><item id='0x461ecc'><name>Zonneboiler</name><value>150.0°C</value></item><item id='0x4817a4'><name>Externe energiebron</name><value>5.0°C</value></item><item id='0x4646b4'><name>Aanvoer max.</name><value>66.0°C</value></item><item id='0x45e76c'><name>Zuiggasleiding comp.</name><value>19.4°C</value></item><item id='0x4607d4'><name>Comp. verwarming</name><value>37.7°C</value></item><item id='0x43e60c'><name>Oververhitting</name><value>4.8 K</value></item><name>Temperaturen</name></Content>`

		// act
		_, err := client.getItemFromResponse("BestaatNiet", []byte(response))

		assert.NotNil(t, err)
	})

	t.Run("ReturnsValueForItemWithoutUnitForPressure", func(t *testing.T) {

		client := client{}

		response := `<Content><item id='0x4e7944'><name>ASD</name><value>Aan</value></item><item id='0x4ffbfc'><name>EVU</name><value>Aan</value></item><item id='0x4ef3b4'><name>HD</name><value>Uit</value></item><item id='0x4dac64'><name>MOT</name><value>Aan</value></item><item id='0x4ca4c4'><name>SWT</name><value>Uit</value></item><item id='0x4fa864'><name>Analoog-In 21</name><value>0.00 V</value></item><item id='0x4d5f1c'><name>Analoog-In 22</name><value>0.00 V</value></item><item id='0x4e6a3c'><name>HD</name><value>8.10 bar</value></item><item id='0x4ca47c'><name>ND</name><value>8.38 bar</value></item><item id='0x4e8004'><name>Debiet</name><value>1200 l/h</value></item><name>Ingangen</name></Content>`

		// act
		value, err := client.getItemFromResponse("HD", []byte(response))

		assert.Nil(t, err)
		assert.Equal(t, float64(8.10), value)
	})
}

func TestGetNavigationFromResponse(t *testing.T) {
	t.Run("ReturnsValueForItemWithoutUnit", func(t *testing.T) {

		client := client{}

		response := `<Navigation id='0x45cd88'><item id='0x45e068'><name>Informatie</name><item id='0x45df90'><name>Temperaturen</name></item><item id='0x455968'><name>Ingangen</name></item><item id='0x455760'><name>Uitgangen</name></item><item id='0x45bf10'><name>Aflooptijden</name></item><item id='0x456f08'><name>Bedrijfsuren</name></item><item id='0x4643a8'><name>Storingsbuffer</name></item><item id='0x3ddfa8'><name>Afschakelingen</name></item><item id='0x45d840'><name>Installatiestatus</name></item><item id='0x460cb8'><name>Energie</name></item><item id='0x4586a8'><name>GBS</name></item></item><item id='0x450798'><name>Instelling</name><item id='0x460bd0'><name>Bedrijfsmode</name></item><item id='0x461170'><name>Temperaturen</name></item><item id='0x462988'><name>Systeeminstelling</name></item></item><item id='0x3dc420'><name>Klokprogramma</name><readOnly>true</readOnly><item id='0x453560'><name>Verwarmen</name><readOnly>true</readOnly><item id='0x45e118'><name>Week</name></item><item id='0x45df00'><name>5+2</name></item><item id='0x45c200'><name>Dagen (Ma, Di,...)</name></item></item><item id='0x43e8e8'><name>Warmwater</name><readOnly>true</readOnly><item id='0x4642a8'><name>Week</name></item><item id='0x463940'><name>5+2</name></item><item id='0x463b68'><name>Dagen (Ma, Di,...)</name></item></item><item id='0x3dcc00'><name>Zwembad</name><readOnly>true</readOnly><item id='0x455580'><name>Week</name></item><item id='0x463f78'><name>5+2</name></item><item id='0x462690'><name>Dagen (Ma, Di,...)</name></item></item></item><item id='0x45c7b0'><name>Toegang: Gebruiker</name></item></Navigation>`

		// act
		navigation, err := client.getNavigationFromResponse([]byte(response))

		assert.Nil(t, err)
		assert.Equal(t, 4, len(navigation.Items))
		assert.Equal(t, "Informatie", navigation.Items[0].Name)
		assert.Equal(t, 10, len(navigation.Items[0].Items))
		assert.Equal(t, "Temperaturen", navigation.Items[0].Items[0].Name)
		assert.Equal(t, "0x45df90", navigation.Items[0].Items[0].ID)
	})
}
