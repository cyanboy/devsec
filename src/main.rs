mod datadog;
mod db;
mod gitlab;
mod repositories;
mod statistics;
mod utils;
mod vulnerabilities;

use crate::{
    datadog::process_csv, db::init_db, gitlab::updater::GitLabUpdaterService,
    repositories::search_repositories, statistics::get_repository_statistics,
};
use clap::{Parser, Subcommand};
use sqlx::SqlitePool;
use std::error::Error;
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
    Import {
        #[command(subcommand)]
        service: ImportServices,
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

#[derive(Subcommand)]
enum ImportServices {
    Datadog {
        #[arg(short, long, value_name = "Input file", help = "File to read from")]
        input: String,

        #[arg(short, long, value_name = "GitLab group id")]
        group_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let pool = init_db().await?;

    match cli.command {
        Some(Commands::Update { service }) => update(&pool, service).await?,
        Some(Commands::Import { service }) => import(&pool, service).await?,
        Some(Commands::Stats { json }) => stats(&pool, json).await?,
        Some(Commands::Search {
            query,
            json,
            include_archived,
            limit,
        }) => search(&pool, &query, json, include_archived, limit).await?,
        None => {}
    };

    Ok(())
}

async fn update(pool: &SqlitePool, service: UpdateServices) -> Result<(), Box<dyn Error>> {
    match service {
        UpdateServices::Gitlab { auth, group_id } => {
            let gitlab_updater = GitLabUpdaterService::new(&auth, &group_id);

            if let Err(e) = gitlab_updater.update(pool).await {
                eprintln!(
                    "❌ Error updating GitLab data: {}\n🔍 Cause: {:?}",
                    e,
                    e.source()
                );
            }
        }
    }
    Ok(())
}

async fn import(pool: &SqlitePool, service: ImportServices) -> Result<(), Box<dyn Error>> {
    match service {
        ImportServices::Datadog { input, group_id } => {
            process_csv(pool, &input, &group_id).await?;
        }
    }
    Ok(())
}

async fn stats(pool: &SqlitePool, json: bool) -> Result<(), Box<dyn Error>> {
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
    pool: &SqlitePool,
    query: &str,
    json: bool,
    include_archived: bool,
    limit: i64,
) -> Result<(), Box<dyn Error>> {
    let data = search_repositories(pool, query, include_archived, limit).await?;

    if json {
        println!("{}", serde_json::to_string(&data)?);
    } else {
        let mut table = Table::new(&data);
        table.with(Style::modern());
        println!("{table}");
    }
    Ok(())
}
