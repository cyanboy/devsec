use clap::{Parser, Subcommand};
use devsec::{
    db::init_db,
    gitlab::service::GitLabUpdaterService,
    repositories::{get_most_frequent_languages, search_repositories},
};
use std::error::Error;

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
    Stats,
    Search {
        #[arg(short, long, value_name = "search query")]
        query: String,

        #[arg(long, help = "Return result as json")]
        json: bool,

        #[arg(long, help = "Include archived repositories in search results")]
        include_archived: bool,
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
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let pool = init_db().await?;

    match cli.command {
        Some(Commands::Update { service }) => handle_update(service, pool).await?,
        Some(Commands::Stats) => handle_stats(&pool).await?,
        Some(Commands::Search {
            query,
            json,
            include_archived,
        }) => handle_search(&pool, &query, json, include_archived).await?,
        None => {}
    };

    Ok(())
}

async fn handle_update(
    service: UpdateServices,
    pool: sqlx::SqlitePool,
) -> Result<(), Box<dyn Error>> {
    match service {
        UpdateServices::Gitlab { auth, group_id } => {
            let gitlab_updater = GitLabUpdaterService::new(&auth, &group_id, pool);

            if let Err(e) = gitlab_updater.update().await {
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

async fn handle_stats(pool: &sqlx::SqlitePool) -> Result<(), Box<dyn Error>> {
    let most_used = get_most_frequent_languages(pool).await?;
    for lang in most_used {
        println!("{}: {:.2}%", lang.0, lang.1);
    }
    Ok(())
}

async fn handle_search(
    pool: &sqlx::SqlitePool,
    query: &str,
    json: bool,
    include_archived: bool,
) -> Result<(), Box<dyn Error>> {
    let result = search_repositories(pool, query, include_archived).await?;

    if json {
        println!("{}", serde_json::to_string(&result)?);
    } else {
        for repo in result {
            println!("{} {} {}", repo.namespace, repo.name, repo.web_url);
        }
    }
    Ok(())
}
