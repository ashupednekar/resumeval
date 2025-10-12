CREATE TABLE IF NOT EXISTS evaluations (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    job_id INTEGER NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    created_by VARCHAR(50) NOT NULL,
    status VARCHAR(50) DEFAULT 'pending',
    total_resumes INTEGER DEFAULT 0,
    processed INTEGER DEFAULT 0,
    accepted INTEGER DEFAULT 0,
    rejected INTEGER DEFAULT 0,
    pending INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- TODO: To reuse resume embeddings, decouple resume from evals
CREATE TABLE IF NOT EXISTS resumes (
    id SERIAL PRIMARY KEY,
    evaluation_id INTEGER NOT NULL REFERENCES evaluations(id) ON DELETE CASCADE,
    filename VARCHAR(255) NOT NULL,
    original_filename VARCHAR(255) NOT NULL,
    file_path TEXT NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    status VARCHAR(50) DEFAULT 'pending',
    score VARCHAR(5),
    feedback TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    embedding vector(768)
);

CREATE INDEX idx_evaluations_job_id ON evaluations(job_id);
CREATE INDEX idx_evaluations_created_by ON evaluations(created_by);
CREATE INDEX idx_evaluations_status ON evaluations(status);
CREATE INDEX idx_resumes_evaluation_id ON resumes(evaluation_id);
CREATE INDEX idx_resumes_status ON resumes(status);
