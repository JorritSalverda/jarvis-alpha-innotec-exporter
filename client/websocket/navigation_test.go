package websocket

import (
	"encoding/xml"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestMarshal(t *testing.T) {
	t.Run("ReturnsXmlString", func(t *testing.T) {

		navigation := getNavigation()

		// act
		xmlString, err := xml.Marshal(navigation)

		assert.Nil(t, err)
		assert.Equal(t, "<Navigation id=\"0x45cd88\"><item id=\"0x45df90\"><name>Informatie</name><item id=\"0x45df90\"><name>Temperaturen</name></item><item id=\"0x455968\"><name>Ingangen</name></item></item><item id=\"0x450798\"><name>Instelling</name></item><item id=\"0x3dc420\"><name>Klokprogramma</name></item><item id=\"0x45c7b0\"><name>Toegang: Gebruiker</name></item></Navigation>", string(xmlString))
	})
}

func TestUnmarshal(t *testing.T) {
	t.Run("ReturnsXmlString", func(t *testing.T) {

		var navigation Navigation
		xmlString := "<Navigation id=\"0x45cd88\"><item id=\"0x45df90\"><name>Informatie</name><item id=\"0x45df90\"><name>Temperaturen</name></item><item id=\"0x455968\"><name>Ingangen</name></item></item><item id=\"0x450798\"><name>Instelling</name></item><item id=\"0x3dc420\"><name>Klokprogramma</name></item><item id=\"0x45c7b0\"><name>Toegang: Gebruiker</name></item></Navigation>"

		// act
		err := xml.Unmarshal([]byte(xmlString), &navigation)

		assert.Nil(t, err)
		assert.Equal(t, 4, len(navigation.Items))
		assert.Equal(t, "Informatie", navigation.Items[0].Name)
		assert.Equal(t, 2, len(navigation.Items[0].Items))
		assert.Equal(t, "Temperaturen", navigation.Items[0].Items[0].Name)
		assert.Equal(t, "0x45df90", navigation.Items[0].Items[0].ID)
		assert.Equal(t, "Ingangen", navigation.Items[0].Items[1].Name)
		assert.Equal(t, "0x455968", navigation.Items[0].Items[1].ID)
	})
}

func TestGetNavigationItemID(t *testing.T) {
	t.Run("ReturnsItemAtTopLevel", func(t *testing.T) {

		navigation := getNavigation()

		// act
		itemID, err := navigation.GetNavigationItemID("Informatie")

		assert.Nil(t, err)
		assert.Equal(t, "0x45df90", itemID)
	})

	t.Run("ReturnsItemNestedInsideTopLevelItem", func(t *testing.T) { // <Navigation id='0x45cd88'><item id='0x45e068'><name>Informatie</name><item id='0x45df90'><name>Temperaturen</name></item><item id='0x455968'><name>Ingangen</name></item><item id='0x455760'><name>Uitgangen</name></item><item id='0x45bf10'><name>Aflooptijden</name></item><item id='0x456f08'><name>Bedrijfsuren</name></item><item id='0x4643a8'><name>Storingsbuffer</name></item><item id='0x3ddfa8'><name>Afschakelingen</name></item><item id='0x45d840'><name>Installatiestatus</name></item><item id='0x460cb8'><name>Energie</name></item><item id='0x4586a8'><name>GBS</name></item></item><item id='0x450798'><name>Instelling</name><item id='0x460bd0'><name>Bedrijfsmode</name></item><item id='0x461170'><name>Temperaturen</name></item><item id='0x462988'><name>Systeeminstelling</name></item></item><item id='0x3dc420'><name>Klokprogramma</name><readOnly>true</readOnly><item id='0x453560'><name>Verwarmen</name><readOnly>true</readOnly><item id='0x45e118'><name>Week</name></item><item id='0x45df00'><name>5+2</name></item><item id='0x45c200'><name>Dagen (Ma, Di,...)</name></item></item><item id='0x43e8e8'><name>Warmwater</name><readOnly>true</readOnly><item id='0x4642a8'><name>Week</name></item><item id='0x463940'><name>5+2</name></item><item id='0x463b68'><name>Dagen (Ma, Di,...)</name></item></item><item id='0x3dcc00'><name>Zwembad</name><readOnly>true</readOnly><item id='0x455580'><name>Week</name></item><item id='0x463f78'><name>5+2</name></item><item id='0x462690'><name>Dagen (Ma, Di,...)</name></item></item></item><item id='0x45c7b0'><name>Toegang: Gebruiker</name></item></Navigation>

		navigation := getNavigation()

		// act
		itemID, err := navigation.GetNavigationItemID("Informatie > Ingangen")

		assert.Nil(t, err)
		assert.Equal(t, "0x455968", itemID)
	})
}

func getNavigation() Navigation {

	// <Navigation id='0x45cd88'><item id='0x45e068'><name>Informatie</name><item id='0x45df90'><name>Temperaturen</name></item><item id='0x455968'><name>Ingangen</name></item><item id='0x455760'><name>Uitgangen</name></item><item id='0x45bf10'><name>Aflooptijden</name></item><item id='0x456f08'><name>Bedrijfsuren</name></item><item id='0x4643a8'><name>Storingsbuffer</name></item><item id='0x3ddfa8'><name>Afschakelingen</name></item><item id='0x45d840'><name>Installatiestatus</name></item><item id='0x460cb8'><name>Energie</name></item><item id='0x4586a8'><name>GBS</name></item></item><item id='0x450798'><name>Instelling</name><item id='0x460bd0'><name>Bedrijfsmode</name></item><item id='0x461170'><name>Temperaturen</name></item><item id='0x462988'><name>Systeeminstelling</name></item></item><item id='0x3dc420'><name>Klokprogramma</name><readOnly>true</readOnly><item id='0x453560'><name>Verwarmen</name><readOnly>true</readOnly><item id='0x45e118'><name>Week</name></item><item id='0x45df00'><name>5+2</name></item><item id='0x45c200'><name>Dagen (Ma, Di,...)</name></item></item><item id='0x43e8e8'><name>Warmwater</name><readOnly>true</readOnly><item id='0x4642a8'><name>Week</name></item><item id='0x463940'><name>5+2</name></item><item id='0x463b68'><name>Dagen (Ma, Di,...)</name></item></item><item id='0x3dcc00'><name>Zwembad</name><readOnly>true</readOnly><item id='0x455580'><name>Week</name></item><item id='0x463f78'><name>5+2</name></item><item id='0x462690'><name>Dagen (Ma, Di,...)</name></item></item></item><item id='0x45c7b0'><name>Toegang: Gebruiker</name></item></Navigation>

	return Navigation{
		ID: "0x45cd88",
		Items: []NavigationItem{
			{
				ID:   "0x45df90",
				Name: "Informatie",
				Items: []NavigationItem{
					{
						ID:   "0x45df90",
						Name: "Temperaturen",
					},
					{
						ID:   "0x455968",
						Name: "Ingangen",
					},
				},
			},
			{
				ID:   "0x450798",
				Name: "Instelling",
			},
			{
				ID:   "0x3dc420",
				Name: "Klokprogramma",
			},
			{
				ID:   "0x45c7b0",
				Name: "Toegang: Gebruiker",
			},
		},
	}

}
