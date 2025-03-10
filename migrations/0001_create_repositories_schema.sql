CREATE TABLE repositories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    external_id INTEGER NOT NULL,
    source TEXT NOT NULL,
    path TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    pushed_at TEXT NOT NULL,
    ssh_url TEXT NOT NULL,
    web_url TEXT NOT NULL,
    private BOOLEAN NOT NULL CHECK (private IN (0, 1)) DEFAULT 0,
    forks_count INTEGER NOT NULL,
    archived BOOLEAN NOT NULL CHECK (archived IN (0, 1)) DEFAULT 0,
    size INTEGER NOT NULL,
    commit_count INTEGER NOT NULL,
    UNIQUE (external_id, source)
);