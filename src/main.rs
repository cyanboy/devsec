mod domain;
mod error;
mod infrastructure;
mod repository;
mod service;

use clap::{Parser, Subcommand};
use domain::statistics::get_repository_statistics;
use error::AppError;
use infrastructure::{api::gitlab::client::GitLabClient, db::connection::init_db};
use repository::codebase_repository::CodebaseRepository;
use service::codebase_service::CodebaseService;
use sqlx::SqlitePool;
use tabled::{
    Table,
    settings::{Rotate, Style},
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Update {
        #[command(subcommand)]
        service: UpdateServices,
    },
    Stats {
        #[arg(long, help = "Return stats as JSON")]
        json: bool,
    },
    Search {
        #[arg(short, long, value_name = "search query")]
        query: String,

        #[arg(long, help = "Return result as json")]
        json: bool,

        #[arg(long, help = "Include archived repositories in search results")]
        include_archived: bool,

        #[arg(
            short = 'n',
            long,
            default_value_t = 10,
            help = "Limit the number of search results"
        )]
        limit: i64,
    },
}

#[derive(Subcommand)]
enum UpdateServices {
    Gitlab {
        #[arg(long, value_name = "GITLAB_TOKEN", env = "GITLAB_TOKEN")]
        auth: String,

        #[arg(short, long, value_name = "GitLab group id")]
        group_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let cli = Cli::parse();
    let pool = init_db().await?;

    let codebase_repository = CodebaseRepository::new(&pool);

    match cli.command {
        Some(Commands::Update { service }) => update(codebase_repository, service).await?,
        Some(Commands::Stats { json }) => stats(&pool, json).await?,
        Some(Commands::Search {
            query,
            json,
            include_archived,
            limit,
        }) => search(codebase_repository, &query, json, include_archived, limit).await?,
        None => {}
    };

    Ok(())
}

async fn update(
    codebase_repository: CodebaseRepository<'_>,
    service: UpdateServices,
) -> Result<(), AppError> {
    match service {
        UpdateServices::Gitlab { auth, group_id } => {
            let gitlab_client = GitLabClient::new(&auth);
            let codebase_service = CodebaseService::new(codebase_repository, gitlab_client);
            codebase_service.update_from_gitlab(&group_id).await?;
        }
    }
    Ok(())
}

async fn stats(pool: &SqlitePool, json: bool) -> Result<(), AppError> {
    let data = get_repository_statistics(pool).await?;

    if json {
        println!("{}", serde_json::to_string(&data)?);
    } else {
        let mut table = Table::new(vec![&data]);
        table.with(Style::modern());
        table.with((Rotate::Left, Rotate::Top));
        println!("{table}");
    }

    Ok(())
}

async fn search(
    codebase_repository: CodebaseRepository<'_>,
    query: &str,
    json: bool,
    include_archived: bool,
    limit: i64,
) -> Result<(), AppError> {
    let gitlab_client = GitLabClient::new("");
    let codebase_service = CodebaseService::new(codebase_repository, gitlab_client);
    let data = codebase_service
        .search(query, include_archived, limit)
        .await?;

    if json {
        println!("{}", serde_json::to_string(&data)?);
    } else {
        let mut table = Table::new(&data);
        table.with(Style::modern());
        println!("{table}");
    }
    Ok(())
}
