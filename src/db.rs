use directories::ProjectDirs;
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::{error::Error, str::FromStr};

pub async fn init_db() -> Result<SqlitePool, Box<dyn Error>> {
    let proj_dirs = match ProjectDirs::from("", "", "devsec") {
        Some(proj_dirs) => proj_dirs,
        None => {
            eprintln!("Could not find home directory");
            std::process::exit(1);
        }
    };

    let data_dir = proj_dirs.data_dir();

    std::fs::create_dir_all(data_dir)?;

    let db_url = if cfg!(debug_assertions) {
        "sqlite://devsec.db".to_string()
    } else {
        format!("sqlite://{}/devsec.db", data_dir.display())
    };

    let opts = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}
