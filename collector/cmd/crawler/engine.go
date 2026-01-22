package main

// AIEngine encapsules how we process text (Volatile)
type AIEngine interface {
	ProcessText(rawText string) (string, error)
}

// SimpleTextEngine implements AIEngine with basic string manipulation
type SimpleTextEngine struct{}

func NewSimpleTextEngine() *SimpleTextEngine {
	return &SimpleTextEngine{}
}

func (e *SimpleTextEngine) ProcessText(rawText string) (string, error) {
	// In the future, this calls an LLM to summarize or extracting keywords.
	// For now, it just passes through or does basic cleaning.
	return rawText, nil
}
