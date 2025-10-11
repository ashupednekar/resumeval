CREATE TABLE IF NOT EXISTS jobs (
    id SERIAL PRIMARY KEY,
    create_by VARCHAR(50) NOT NULL,
    title VARCHAR(255) NOT NULL,
    department VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    requirements TEXT NOT NULL,
    url VARCHAR(512),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_jobs_department ON jobs(department);
CREATE INDEX idx_jobs_title ON jobs(title);

