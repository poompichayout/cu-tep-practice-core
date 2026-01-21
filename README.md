# CU-TEP Practice Core API

An AI-powered backend system designed to personalize CU-TEP exam practice. This project uses a **RAG (Retrieval-Augmented Generation)** approach to gather practice materials from the web, understand them using **Google Gemini**, and generate infinite practice tests tailored to the user.

## 🏗 Architecture

The system consists of three main components:

1.  **Core API (Rust)**: The brain of the operation.
    *   **Framework**: Axum (Fast, Async).
    *   **Role**: Handles data ingestion, manages the RAG pipeline, and integrates with Google Gemini for text analysis and question generation.
    *   **Database**: PostgreSQL with `pgvector` for storing content and vector embeddings.
2.  **Collector (Go)**: The efficient gatherer.
    *   **Libraries**: Colly (for static sites) & Chromedp (for dynamic).
    *   **Role**: Crawls target websites to harvest CU-TEP practice questions and reading passages, feeding them to the Core API.
3.  **AI Engine (Gemini)**: The intelligence.
    *   **Model**: Gemini 1.5 Pro/Flash.
    *   **Role**: Extracts structured questions from raw scraped text and generates new questions based on retrieved context.

## 🚀 Tech Stack

*   **Backend**: Rust (Axum, SQLx, Tokio)
*   **Collector**: Go (Colly)
*   **Database**: PostgreSQL + pgvector
*   **AI**: Google Gemini API
*   **Testing**: Bruno (API Client)

## 🛠️ Prerequisites

*   **Docker & Docker Compose**: For the database.
*   **Rust**: `cargo` and `rustc` (latest stable).
*   **Go**: Version 1.21 or higher.

## 🏃‍♂️ Getting Started

### 1. Start the Database
Spin up the PostgreSQL instance with `pgvector` enabled.
```bash
docker-compose up -d
```

### 2. Configure Environment
Ensure you have a `.env` file in the root directory:
```env
DATABASE_URL=postgres://user:password@localhost:5432/cutep
GEMINI_API_KEY=your_actual_api_key_here
```

### 3. Run the Core API (Rust)
This service handles ingestion and processing.
```bash
cd backend
cargo run
```
*The server will start listening on `0.0.0.0:8080`.*
*(Note: Database migrations run automatically on startup).*

### 4. Run the Collector (Go)
Open a new terminal to run the scraper.
```bash
cd collector
go mod tidy
go run cmd/crawler/main.go --url "https://example.com/some-practice-page"
```
*This will scrape the URL and send the raw content to the Core API for processing.*

## 🧪 API Testing (Bruno)

We use **Bruno** for API experimentation and testing.
1.  Install [Bruno](https://www.usebruno.com/).
2.  Open the `bruno/` folder located in this project root.
3.  You will find pre-configured requests like **Health Check** and **Ingest Data**.

## 📂 Project Structure

```
.
├── backend/            # Rust Core API (Axum)
│   ├── src/
│   │   ├── api/        # REST Endpoints
│   │   ├── core/       # Business Logic (Gemini, Processor)
│   │   └── db/         # Database Connection & Models
│   └── migrations/     # SQLx Migrations
├── collector/          # Go Scraper (Colly)
│   └── cmd/crawler/    # Crawler CLI entry point
├── bruno/              # API Collection for testing
└── docker-compose.yml  # Infrastructure setup
```
