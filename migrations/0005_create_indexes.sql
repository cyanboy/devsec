CREATE INDEX idx_repository_languages_repo_id ON repository_languages (repository_id);
CREATE INDEX idx_repository_languages_lang_id ON repository_languages (language_id);
CREATE INDEX idx_vulnerabilities_cve ON vulnerabilities (cve);
CREATE INDEX idx_repository_vuln_repo_id ON repository_vulnerability (repository_id);
CREATE INDEX idx_repository_vuln_vuln_id ON repository_vulnerability (vulnerability_id);