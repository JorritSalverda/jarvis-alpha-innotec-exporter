package main

import (
	"encoding/binary"
	"fmt"
	"log"
	"os"
	"time"

	"github.com/goburrow/modbus"
)

// HeatpumpClient is the interface for connecting to alpha innotec heatpump via ethernet
type HeatpumpClient interface {
	GetTotalWhOut() (totalWhOut uint64, err error)
}

// NewHeatpumpClient returns new HeatpumpClient
func NewHeatpumpClient(host string, port int, unitID int) (HeatpumpClient, error) {
	if host == "" {
		return nil, fmt.Errorf("Please set the ip address of your Alpha Innotec heatpump on your local network")
	}
	if port != 502 && (port < 49152 || port > 65535) {
		return nil, fmt.Errorf("Please set the modbus port of your Alpha Innotec heatpump on your local network to its default 502, or anywhere between 49152 and 65535 if changed in the installer menu")
	}

	return &heatpumpClientImpl{
		host: host,
		port: port,
	}, nil
}

type heatpumpClientImpl struct {
	host   string
	port   int
	unitID int
}

func (c *heatpumpClientImpl) GetTotalWhOut() (totalWhOut uint64, err error) {

	// Modbus TCP
	handler := modbus.NewTCPClientHandler(fmt.Sprintf("%v:%v", c.host, c.port))
	handler.Timeout = 20 * time.Second
	handler.SlaveId = 0x3
	handler.Logger = log.New(os.Stdout, "test: ", log.LstdFlags)
	// Connect manually so that multiple requests are handled in one connection session
	err = handler.Connect()
	if err != nil {
		return
	}
	defer handler.Close()
	client := modbus.NewClient(handler)

	// Read input register (see https://files.sma.de/downloads/MODBUS-HTML_STP8.0-10.0-3AV-40_GG10_V10.zip)
	totalWhOutBytes, err := client.ReadHoldingRegisters(30513, 4)
	if err != nil {
		return
	}

	totalWhOut = binary.BigEndian.Uint64(totalWhOutBytes)

	return
}
