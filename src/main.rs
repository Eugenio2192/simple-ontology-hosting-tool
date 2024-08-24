use actix_files::NamedFile;
use actix_web::web::Data;
use actix_web::{
    dev::Server, error, get, http::StatusCode, web, App, HttpRequest, HttpResponse, HttpServer,
    Responder, Result,
};
use derive_more::{Display, Error};
use soht::database::{build_database, get_connection_pool, initialize_tables, insert_content};
use soht::splitter::split_ontology;
use sqlx::SqlitePool;
#[derive(Debug, Display, Error)]
#[display(fmt = "Not found")]
struct NotFoundError;

impl error::ResponseError for NotFoundError {
    fn error_response(&self) -> HttpResponse {
        not_found()
    }
    fn status_code(&self) -> StatusCode {
        StatusCode::NOT_FOUND
    }
}
use std::{net::TcpListener, path::PathBuf};

pub struct Application {
    _port: u16,
    server: Server,
}

struct Content {
    _name: String,
    content: Vec<u8>,
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[get("/ontology/{identifier}")]
async fn identifier(req: HttpRequest) -> Result<impl Responder, NotFoundError> {
    let base = req.match_info().query("identifier");
    let mut identifier: PathBuf = PathBuf::new();
    identifier.push("ontology");
    identifier.push(format!("{}.xml", base));
    NamedFile::open(identifier).map_err(|_| NotFoundError)
}

async fn query_content(pool: &SqlitePool, base: &str) -> Result<Option<Content>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT name, content
        FROM xml_cache
        WHERE name = $1
        "#,
        base,
    )
    .fetch_optional(pool)
    .await?;
    Ok(result.map(|r| Content {
        _name: r.name,
        content: r.content,
    }))
}

#[get("/ontology/{identifier}")]
async fn get_identifier(req: HttpRequest, pool: web::Data<SqlitePool>) -> HttpResponse {
    let base = req.match_info().query("identifier");
    let content = query_content(&pool, base).await;
    match content {
        Ok(c) => c.map_or(not_found(), |cont| {
            HttpResponse::Ok()
                .content_type("text/xml")
                .body(cont.content)
        }),
        Err(_) => not_found(),
    }
}

fn not_found() -> HttpResponse {
    HttpResponse::build(StatusCode::NOT_FOUND)
        .content_type("text/html; charset=utf-8")
        .body("<h1>Error 404</h>")
}
async fn default_service() -> HttpResponse {
    not_found()
}
impl Application {
    pub async fn build(connection_pool: SqlitePool, port: u16) -> Result<Self, anyhow::Error> {
        let address = TcpListener::bind(format!("127.0.0.1:{}", port))?;
        let pool = Data::new(connection_pool);
        let server = HttpServer::new(move || {
            App::new()
                .service(get_identifier)
                .route("/health_check", web::get().to(health_check))
                .default_service(web::route().to(default_service))
                .app_data(pool.clone())
        })
        .listen(address)?
        .run();
        Ok(Application {
            _port: port,
            server,
        })
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    build_database().await?;
    let conncection_pool = get_connection_pool().await;
    initialize_tables(&conncection_pool).await?;
    insert_content("EventOntology", &conncection_pool).await?;
    match split_ontology("ontology/EventOntology.xml", &conncection_pool).await {
        Ok(_) => (),
        Err(e) => println!("{:?}", e),
    };
    let application = Application::build(conncection_pool, 8000).await?;
    let handle = tokio::spawn(application.run_until_stopped());
    let _result = handle.await?;
    Ok(())
}
