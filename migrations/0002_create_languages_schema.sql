CREATE TABLE programming_languages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL
);

CREATE TABLE codebase_languages (
    codebase_id INTEGER NOT NULL,
    language_id INTEGER NOT NULL,
    percentage REAL NOT NULL,
    PRIMARY KEY (codebase_id, language_id),
    FOREIGN KEY (codebase_id) REFERENCES codebases (id) ON DELETE CASCADE,
    FOREIGN KEY (language_id) REFERENCES programming_languages (id) ON DELETE CASCADE
);