package main

import (
	"strings"

	"github.com/gocolly/colly/v2"
)

// ResourceAccessor interface encapsulates where data comes from (Volatile)
type ResourceAccessor interface {
	FetchContent(url string) (string, error)
}

// WebScraperAccessor implements ResourceAccessor for web pages
type WebScraperAccessor struct{}

func NewWebScraperAccessor() *WebScraperAccessor {
	return &WebScraperAccessor{}
}

func (w *WebScraperAccessor) FetchContent(url string) (string, error) {
	// Initialize Colly
	// Note: In a real scenario, we might inject colly collector or options here
	c := colly.NewCollector(
		colly.AllowedDomains(extractDomain(url)),
	)

	var contentBuilder strings.Builder

	// On every HTML element which looks like text ...
	c.OnHTML("body", func(e *colly.HTMLElement) {
		// Simple text extraction for MVP.
		text := strings.TrimSpace(e.Text)
		contentBuilder.WriteString(text)
	})

	err := c.Visit(url)
	if err != nil {
		return "", err
	}

	return contentBuilder.String(), nil
}

func extractDomain(u string) string {
	// Simple domain extractor (naive)
	parts := strings.Split(u, "/")
	if len(parts) > 2 {
		return parts[2]
	}
	return ""
}
