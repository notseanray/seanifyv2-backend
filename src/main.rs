mod api;
mod extractors;
mod middlewares;
mod types;

use actix_cors::Cors;
use actix_web::{App, HttpServer};
use dotenv::dotenv;

use async_once::AsyncOnce;
use lazy_static::lazy_static;
use log::{error, info};
use sqlx::{Pool, Sqlite};
use std::{error::Error, time::Duration};

lazy_static! {
    pub(crate) static ref DB: AsyncOnce<Pool<Sqlite>> =
        AsyncOnce::new(async { Database::setup(env!("DATABASE_URL"), 100).await.expect("failed to check database url") });
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
    pub(crate) async fn setup(uri: &str, timeout: u64) -> Result<Pool<Sqlite>, Box<dyn Error>> {
        Ok(Self::try_connect(uri, timeout).await)
    }

    pub async fn try_connect(uri: &str, timeout: u64) -> Pool<Sqlite> {
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

    async fn connect(uri: &str) -> Result<Pool<Sqlite>, Box<dyn Error>> {
        // let connect_opts = SqliteConnectOptions::new().create_if_missing(true);
        Ok(Pool::<Sqlite>::connect(uri).await?)
    }
}
