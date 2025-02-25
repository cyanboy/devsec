use clap::{Parser, Subcommand};
use devsec::{
    db::init_db,
    gitlab::service::GitLabUpdaterService,
    repositories::{get_most_frequent_languages, search_repositories},
};
use std::error::Error;
use tabled::{
    Table,
    settings::{
        Alignment, Style,
        object::{Columns, Object, Rows, Segment},
    },
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
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let pool = init_db().await?;

    match cli.command {
        Some(Commands::Update { service }) => update(service, pool).await?,
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

async fn update(service: UpdateServices, pool: sqlx::SqlitePool) -> Result<(), Box<dyn Error>> {
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

async fn stats(pool: &sqlx::SqlitePool, json: bool) -> Result<(), Box<dyn Error>> {
    let most_used = get_most_frequent_languages(pool).await?;
    for lang in most_used {
        println!("{}: {:.2}%", lang.0, lang.1);
    }
    Ok(())
}

async fn search(
    pool: &sqlx::SqlitePool,
    query: &str,
    json: bool,
    include_archived: bool,
    limit: i64,
) -> Result<(), Box<dyn Error>> {
    let result = search_repositories(pool, query, include_archived, limit).await?;

    if json {
        println!("{}", serde_json::to_string(&result)?);
    } else {
        let mut table = Table::new(&result);
        table.with(Style::modern());
        print!("{}", table.to_string());
    }
    Ok(())
}
