-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Table to store raw ingested content (e.g., HTML body, text)
CREATE TABLE IF NOT EXISTS raw_materials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    url TEXT NOT NULL,
    content TEXT NOT NULL,
    source_type TEXT NOT NULL, -- 'web', 'pdf', etc.
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed BOOLEAN DEFAULT FALSE
);

-- Table to store structured questions extracted from raw materials
CREATE TABLE IF NOT EXISTS questions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    raw_material_id UUID REFERENCES raw_materials(id),
    topic TEXT NOT NULL, -- 'reading', 'error_id', 'listening'
    content JSONB NOT NULL, -- Stores the structured question data
    difficulty_level TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table for vector embeddings (RAG)
CREATE TABLE IF NOT EXISTS embeddings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    question_id UUID REFERENCES questions(id),
    chunk_text TEXT NOT NULL, -- The specific text chunk embedded
    embedding vector(768), -- Gemini 1.5 Pro/Flash typically uses 768 dimensions (check specific model)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX ON embeddings USING hnsw (embedding vector_cosine_ops);
