CREATE TABLE codebases (
    id INTEGER PRIMARY KEY,
    repo_name TEXT NOT NULL,
    full_name TEXT NOT NULL,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
    pushed_at TIMESTAMPTZ,
    ssh_url TEXT,
    web_url TEXT,
    private BOOLEAN,
    forks_count INTEGER NOT NULL,
    archived BOOLEAN 
);

CREATE TABLE languages (
    id SERIAL PRIMARY KEY,
    language_name TEXT UNIQUE NOT NULL
);

CREATE TABLE codebase_languages (
    codebase_id INTEGER NOT NULL REFERENCES codebases(id) ON DELETE CASCADE,
    language_id INTEGER NOT NULL REFERENCES languages(id) ON DELETE CASCADE,
    percentage REAL NOT NULL CHECK (percentage >= 0.00 AND percentage <= 100.00),
    PRIMARY KEY (codebase_id, language_id)
);