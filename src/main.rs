use clap::{Parser, Subcommand};
use devsec::{
    db::init_db, gitlab::service::GitLabUpdaterService, repositories::service::RepositoryService,
    statistics::service::StatisticsService,
};
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

    let repository_service = RepositoryService::new(&pool);
    let statistics_service = StatisticsService::new(&pool);

    match cli.command {
        Some(Commands::Update { service }) => update(service, repository_service).await?,
        Some(Commands::Stats { json }) => stats(statistics_service, json).await?,
        Some(Commands::Search {
            query,
            json,
            include_archived,
            limit,
        }) => search(repository_service, &query, json, include_archived, limit).await?,
        None => {}
    };

    Ok(())
}

async fn update(
    service: UpdateServices,
    repos: RepositoryService<'_>,
) -> Result<(), Box<dyn Error>> {
    match service {
        UpdateServices::Gitlab { auth, group_id } => {
            let gitlab_updater = GitLabUpdaterService::new(&auth, &group_id, &repos);

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

async fn stats(
    statistics_service: StatisticsService<'_>,
    json: bool,
) -> Result<(), Box<dyn Error>> {
    let data = statistics_service.get_repository_statistics().await?;

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
    repository_service: RepositoryService<'_>,
    query: &str,
    json: bool,
    include_archived: bool,
    limit: i64,
) -> Result<(), Box<dyn Error>> {
    let data = repository_service
        .search_repositories(query, include_archived, limit)
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
