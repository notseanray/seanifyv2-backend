mod api;
mod extractors;
mod middlewares;
mod types;

use actix_web::{App, HttpServer};
use dotenv::dotenv;
use actix_cors::Cors;

use std::sync::{Arc, RwLock};
use sqlx::{Pool, Sqlite, sqlite::SqliteConnection, Connection};
use std::{error::Error, time::Duration};
use log::{info, error};
use lazy_static::lazy_static;
use async_once::AsyncOnce;

lazy_static! {
    pub(crate) static ref DB: AsyncOnce<Arc<RwLock<SqliteConnection>>> = AsyncOnce::new(async {Arc::new(RwLock::new(Database::new("test",100).await.unwrap())) });
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    pretty_env_logger::init();
    let config = types::Config::default();
    HttpServer::new(move || {
        let auth0_config = extractors::Auth0Config::default();
        let cors = Cors::permissive();
        App::new()
            .app_data(auth0_config)
            .wrap(cors)
            .wrap(middlewares::err_handlers())
            .wrap(middlewares::security_headers())
            .wrap(middlewares::logger())
            .service(api::routes())
    })
    .bind((config.host, config.port))?
    .run()
    .await
}

pub(crate) struct Database(Pool<Sqlite>);

impl Database {
    pub(crate) async fn new(uri: &str, timeout: u64) -> Result<SqliteConnection, Box<dyn Error>> {
        Ok(Self::try_connect(&uri, timeout).await)
    }

    pub async fn try_connect(uri: &str, timeout: u64) -> SqliteConnection {
        for i in 1..6 {
            match Self::connect(uri).await {
                Ok(v) => return v,
                Err(e) => info!("Failed to connect to {uri} due to {e}, retrying [{i}/5]"),
            };
            std::thread::sleep(Duration::from_millis(timeout));
        }
        error!("could not aquire database after 5 attempts, quiting");
        std::process::exit(1);
    }

    async fn connect(uri: &str) -> Result<SqliteConnection, Box<dyn Error>> {
        // let connect_opts = SqliteConnectOptions::new().create_if_missing(true);
        Ok(SqliteConnection::connect(&uri).await?)
    }
}
