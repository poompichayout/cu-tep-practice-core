# CU-TEP Practice Core - System Design & Specification
## 1. System Overview
The **CU-TEP Practice Core** is an AI-powered backend system designed to generate personalized practice tests for the CU-TEP exam. It utilizes a **Retrieval-Augmented Generation (RAG)** approach, where a **Collector** service scrapes educational content from the web, and a **Core API** processes this content using **Google Gemini** to extract structured questions and generate vector embeddings for retrieval.
## 2. System Architecture
The system follows the **Volatility-Based Decomposition (VBD)** architecture to separate stable business logic from volatile external dependencies.
### 2.1 Components
*   **Collector (Go)**:
    *   **Role**: Volatile component responsible for interfacing with external websites. It handles the differences in HTML structure and scraping logic.
    *   **Pattern**: Acts as a **Resource Accessor** in VBD terms, but implemented as a separate service for scalability.
    *   **Tech**: Go, Colly (static), Chromedp (dynamic).
*   **Core API (Rust)**:
    *   **Role**: Stable component containing the business rules, data management, and coordination logic.
    *   **Tech**: Rust, Axum, SQLx, Tokio.
    *   **Sub-components**:
        *   **Ingestion Manager**: Coordinates the flow of raw data to processing.
        *   **Education Manager**: Manages question generation and retrieval logic.
        *   **Engines**: Volatile wrappers around AI models (Gemini).
        *   **Accessors**: Volatile wrappers around database or external APIs.
*   **Database (PostgreSQL)**:
    *   **Role**: Persistent storage for raw materials, structured questions, and vector embeddings.
    *   **Tech**: PostgreSQL + `pgvector`.
### 2.2 Volatility Analysis (VBD)
| Component | Volatility Type | Strategy |
| :--- | :--- | :--- |
| **Collector** | **High** (Target layouts change frequently) | Encapsulate in separate executable/service. |
| **AI Engine** | **Medium** (Models update, prompts tweek) | Encapsulate in `engines` module behind stable Traits. |
| **Core API** | **Low** (Business rules are stable) | Keep pure and coordinate other components. |
## 3. API Specification
### 3.1 Ingestion
**Endpoint**: `POST /internal/ingest`
**Description**: Receives raw content scraped by the Collector.
**Request Body** (`application/json`):
```json
{
  "url": "https://example.com/practice-test-1",
  "raw_content": "<html>...</html>",
  "source_type": "web"
}
```
| Field | Type | Description |
| :--- | :--- | :--- |
| `url` | String | The source URL of the content. |
| `raw_content` | String | The raw text or HTML body. |
| `source_type` | String | Origin type (e.g., 'web', 'pdf'). |
**Response**:
*   **201 Created**:
    ```json
    {
      "id": "uuid-string",
      "status": "queued"
    }
    ```
*   **500 Internal Server Error**: Database failure.
## 4. User Flow & Data Flow
### 4.1 Content Ingestion Flow
1.  **Trigger**: User (or Cron) runs the Collector CLI with a target URL.
2.  **Scrape**: Collector fetches the page, handling JS or static HTML.
3.  **Send**: Collector sends JSON payload to `Core API` (`/internal/ingest`).
4.  **Persist**: Core API saves raw content to `raw_materials` table.
5.  **Ack**: Core API responds with `201 Created` immediately (Async processing).
6.  **Process (Background)**:
    *   `process_material` task picks up the `raw_material_id`.
    *   Invokes **Gemini Engine** to analyze text and extract questions.
    *   Saves structured data to `questions` table.
    *   Generates embeddings for the content.
    *   Saves embeddings to `embeddings` table.
### 4.2 Question Generation Flow (Future/VBD)
1.  **Request**: User requests a practice test (e.g., "Reading/Grammar").
2.  **Retrieve**: Core API queries `embeddings` using `pgvector` specifically looking for relevant content.
3.  **Generate**: Core API sends retrieved context + User Request to **Gemini**.
4.  **Response**: Gemini generates a new, unique question based on the context.
5.  **Serve**: API returns the generated test to the user.
## 5. Database Schema
### `raw_materials`
Stores the unprocessed scraped content.
- `id`: UUID (PK)
- `url`: TEXT
- `content`: TEXT
- `source_type`: TEXT
- `processed`: BOOLEAN
### `questions`
Stores extracted, structured canonical data.
- `id`: UUID (PK)
- `raw_material_id`: UUID (FK)
- `topic`: TEXT (reading, error_id, listening)
- `content`: JSONB (The structural representation)
- `difficulty_level`: TEXT
### `embeddings`
Stores vector data for RAG.
- `id`: UUID (PK)
- `question_id`: UUID (FK)
- `chunk_text`: TEXT
- `embedding`: VECTOR(768)