package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
)

// IngestionManager coordinates the flow (Stable)
type IngestionManager struct {
	Accessor ResourceAccessor
	Engine   AIEngine
	APIURL   string
}

func NewIngestionManager(accessor ResourceAccessor, engine AIEngine, apiURL string) *IngestionManager {
	return &IngestionManager{
		Accessor: accessor,
		Engine:   engine,
		APIURL:   apiURL,
	}
}

func (m *IngestionManager) Ingest(url string) error {
	fmt.Printf("Manager: Starting ingestion for %s\n", url)

	// 1. Fetch Data (Accessor)
	rawContent, err := m.Accessor.FetchContent(url)
	if err != nil {
		return fmt.Errorf("failed to fetch content: %w", err)
	}

	if len(rawContent) == 0 {
		return fmt.Errorf("no content extracted")
	}

	// 2. Process Data (Engine)
	processedContent, err := m.Engine.ProcessText(rawContent)
	if err != nil {
		return fmt.Errorf("failed to process content: %w", err)
	}

	// 3. Send to Core API
	// Note: We might want a "StorageAccessor" or "APIAccessor" for this too, 
	// but for now keeping it simple within the Manager workflow as the "Output" step.
	return m.sendToCore(url, processedContent)
}

func (m *IngestionManager) sendToCore(sourceURL, content string) error {
	payload := IngestRequest{
		URL:        sourceURL,
		RawContent: content,
		SourceType: "web",
	}

	jsonData, err := json.Marshal(payload)
	if err != nil {
		return err
	}

	resp, err := http.Post(m.APIURL, "application/json", bytes.NewBuffer(jsonData))
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusCreated && resp.StatusCode != http.StatusOK {
		return fmt.Errorf("API returned status: %s", resp.Status)
	}
	
	fmt.Println("Manager: Successfully ingested content to Core API.")
	return nil
}
