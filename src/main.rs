use clap::{Parser, Subcommand};
use devsec::{
    db::init_db,
    gitlab::service::GitLabUpdaterService,
    repositories::{get_most_frequent_languages, search_repositories},
    tui,
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

        #[arg(default_value_t = true, long, help = "Return result as json")]
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

    match &cli.command {
        Some(Commands::Update { service }) => match service {
            UpdateServices::Gitlab { auth, group_id } => {
                let gitlab_updater = GitLabUpdaterService::new(auth, group_id, pool);

                match gitlab_updater.update().await {
                    Ok(_) => (),
                    Err(e) => eprintln!("❌ Error updating GitLab data: {}", e),
                }
            }
        },
        Some(Commands::Stats) => {
            let most_used = get_most_frequent_languages(&pool).await?;
            most_used
                .iter()
                .for_each(|lang| println!("{}: {:.2}%", lang.0, lang.1));
        }
        Some(Commands::Search {
            query,
            json,
            include_archived,
        }) => {
            let result = search_repositories(&pool, query, *include_archived).await?;

            if *json {
                println!("{}", serde_json::to_string(&result)?);
            } else {
                for repo in result {
                    println!("{}", repo.web_url);
                }
            }
        }
        None => {
            color_eyre::install()?;
            let terminal = ratatui::init();
            let app_result = tui::run(terminal);
            ratatui::restore();
            app_result?
        }
    };

    Ok(())
}
