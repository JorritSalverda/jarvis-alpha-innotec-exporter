package websocket

import (
	"encoding/xml"
	"fmt"
	"strings"
)

// <Navigation id='0x45cd88'><item id='0x45e068'><name>Informatie</name><item id='0x45df90'><name>Temperaturen</name></item><item id='0x455968'><name>Ingangen</name></item><item id='0x455760'><name>Uitgangen</name></item><item id='0x45bf10'><name>Aflooptijden</name></item><item id='0x456f08'><name>Bedrijfsuren</name></item><item id='0x4643a8'><name>Storingsbuffer</name></item><item id='0x3ddfa8'><name>Afschakelingen</name></item><item id='0x45d840'><name>Installatiestatus</name></item><item id='0x460cb8'><name>Energie</name></item><item id='0x4586a8'><name>GBS</name></item></item><item id='0x450798'><name>Instelling</name><item id='0x460bd0'><name>Bedrijfsmode</name></item><item id='0x461170'><name>Temperaturen</name></item><item id='0x462988'><name>Systeeminstelling</name></item></item><item id='0x3dc420'><name>Klokprogramma</name><readOnly>true</readOnly><item id='0x453560'><name>Verwarmen</name><readOnly>true</readOnly><item id='0x45e118'><name>Week</name></item><item id='0x45df00'><name>5+2</name></item><item id='0x45c200'><name>Dagen (Ma, Di,...)</name></item></item><item id='0x43e8e8'><name>Warmwater</name><readOnly>true</readOnly><item id='0x4642a8'><name>Week</name></item><item id='0x463940'><name>5+2</name></item><item id='0x463b68'><name>Dagen (Ma, Di,...)</name></item></item><item id='0x3dcc00'><name>Zwembad</name><readOnly>true</readOnly><item id='0x455580'><name>Week</name></item><item id='0x463f78'><name>5+2</name></item><item id='0x462690'><name>Dagen (Ma, Di,...)</name></item></item></item><item id='0x45c7b0'><name>Toegang: Gebruiker</name></item></Navigation>

type Navigation struct {
	XMLName xml.Name         `xml:"Navigation"`
	ID      string           `xml:"id,attr"`
	Items   []NavigationItem `xml:"item"`
}

type NavigationItem struct {
	XMLName xml.Name         `xml:"item"`
	ID      string           `xml:"id,attr"`
	Name    string           `xml:"name"`
	Items   []NavigationItem `xml:"item"`
}

func (n *Navigation) GetNavigationItemID(itemPath string) (navigationID string, err error) {

	itemPathParts := strings.Split(itemPath, " > ")
	items := n.Items
	for _, p := range itemPathParts {
		exists := false
		for _, i := range items {
			if p == i.Name {
				exists = true
				navigationID = i.ID
				items = i.Items
				break
			}
		}

		if !exists {
			return navigationID, fmt.Errorf("Item %v does not exist", p)
		}
	}

	return
}
