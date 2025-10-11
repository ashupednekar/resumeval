CREATE TABLE IF NOT EXISTS projects (
  project_id VARCHAR(50) PRIMARY KEY,
  name VARCHAR(20) NOT NULL UNIQUE,
  description VARCHAR(50) NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'invite_status') THEN
        CREATE TYPE invite_status AS ENUM ('pending', 'accepted', 'expired');
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS project_access (
    invite_id VARCHAR(50) PRIMARY KEY,
    user_id VARCHAR(50) NOT NULL,
    inviter_id VARCHAR(50) NOT NULL,
    project_id VARCHAR(50) NOT NULL,
    status invite_status NOT NULL DEFAULT 'pending',
    expiry TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE,
    CONSTRAINT fk_project FOREIGN KEY (project_id) REFERENCES projects(project_id) ON DELETE CASCADE,
    CONSTRAINT uq_user_project UNIQUE (user_id, project_id)
);
