use serde::Serialize;
use tabled::Tabled;

#[derive(Tabled, Debug, Serialize)]
pub struct RepoStats {
    pub total_repos: i64,
    pub largest_repo: String,
    pub most_active_repo: String,
    pub newest_repository: String,
    pub most_used_language: String,
    pub private_repo_count: i64,
    pub public_repo_count: i64,
}
