package main

import (
	"flag"
	"log"
)

// IngestRequest defined here for Manager to usage (normally would be in a models package)
type IngestRequest struct {
	URL        string `json:"url"`
	RawContent string `json:"raw_content"`
	SourceType string `json:"source_type"`
}

func main() {
	targetURL := flag.String("url", "", "URL to scrape")
	coreAPI := flag.String("api", "http://localhost:8080/internal/ingest", "Core API Endpoint")
	flag.Parse()

	if *targetURL == "" {
		log.Fatal("Please provide a --url")
	}

	// Initialize VBD Components
	accessor := NewWebScraperAccessor()
	engine := NewSimpleTextEngine()
	
	// Initialize Manager
	manager := NewIngestionManager(accessor, engine, *coreAPI)

	// Execute Workflow
	err := manager.Ingest(*targetURL)
	if err != nil {
		log.Fatal(err)
	}
}
