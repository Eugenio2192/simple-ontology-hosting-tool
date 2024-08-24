use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

const DB_URL: &str = "sqlite://sqlite.db";

pub async fn build_database() -> Result<(), anyhow::Error> {
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        println!("Creating database");
        Sqlite::create_database(DB_URL).await?;
    } else {
        println!("Database exists")
    }
    Ok(())
}

pub async fn get_connection_pool() -> SqlitePool {
    SqlitePool::connect(DB_URL).await.unwrap()
}

pub async fn initialize_tables(pool: &SqlitePool) -> Result<(), anyhow::Error> {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let migrations = std::path::Path::new(&crate_dir).join("./migrations");
    sqlx::migrate::Migrator::new(migrations)
        .await?
        .run(pool)
        .await?;
    println!("Perfomrmed migrations");
    Ok(())
}

pub async fn insert_content(name: &str, pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let mut filename: PathBuf = PathBuf::new();
    filename.push("ontology");
    filename.push(format!("{}.xml", name));
    let content = fs::read(filename).await?;
    let uuid = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO xml_cache (id, name, content)
                VALUES ($1, $2, $3)",
        uuid,
        name,
        content
    )
    .execute(pool)
    .await
    .expect("Failed to store content.");
    Ok(())
}
