package main

import (
	"bytes"
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"net/http"
	"strings"

	"github.com/gocolly/colly/v2"
)

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

	fmt.Printf("Scraping %s...\n", *targetURL)

	// Initialize Colly
	c := colly.NewCollector(
		colly.AllowedDomains(extractDomain(*targetURL)),
	)

	var contentBuilder strings.Builder

	// On every HTML element which looks like text ...
	c.OnHTML("body", func(e *colly.HTMLElement) {
		// Simple text extraction for MVP. 
		// In reality, we might want specific selectors or keep HTML structure.
		// For RAG, raw text is often fine if we have a smart chunker/processor (Gemini).
		text := strings.TrimSpace(e.Text)
		contentBuilder.WriteString(text)
	})

	c.OnScraped(func(r *colly.Response) {
		fmt.Println("Finished scraping", r.Request.URL)
		
		rawContent := contentBuilder.String()
		if len(rawContent) == 0 {
			log.Println("Warning: No content extracted.")
			return
		}

		err := sendToCore(*coreAPI, *targetURL, rawContent)
		if err != nil {
			log.Printf("Failed to ingest: %v\n", err)
		} else {
			fmt.Println("Successfully ingested content to Core API.")
		}
	})

	err := c.Visit(*targetURL)
	if err != nil {
		log.Fatal(err)
	}
}

func extractDomain(u string) string {
	// Simple domain extractor (naive)
	parts := strings.Split(u, "/")
	if len(parts) > 2 {
		return parts[2]
	}
	return ""
}

func sendToCore(apiURL, sourceURL, content string) error {
	payload := IngestRequest{
		URL:        sourceURL,
		RawContent: content,
		SourceType: "web",
	}

	jsonData, err := json.Marshal(payload)
	if err != nil {
		return err
	}

	resp, err := http.Post(apiURL, "application/json", bytes.NewBuffer(jsonData))
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusCreated && resp.StatusCode != http.StatusOK {
		return fmt.Errorf("API returned status: %s", resp.Status)
	}

	return nil
}
