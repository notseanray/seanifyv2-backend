mod api;
mod extractors;
mod fuzzy;
mod middlewares;
mod types;
mod youtube;

use actix_cors::Cors;
use actix_web::{App, HttpServer, Scope};
use dotenv::dotenv;

use crate::types::{Config, DownloadCache, SongSearch};
use actix::{Actor, StreamHandler};
use actix_web::{get, web, Error as ActixError, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use async_once::AsyncOnce;
use lazy_static::lazy_static;
use log::{error, info};
use sqlx::{Pool, Postgres};
use std::sync::{Arc, Mutex, RwLock};
use std::{error::Error, time::Duration};

pub(crate) const VERSION: &str = "0.1.0";
pub(crate) const BRANCH: &str = "main";

lazy_static! {
    pub(crate) static ref CONFIG: Config = Config::default();
    pub(crate) static ref DB: AsyncOnce<Arc<Database>> = AsyncOnce::new(async {
        Arc::new(Database {
            db: Database::setup(env!("DATABASE_URL"), 100)
                .await
                .expect("failed to check database url"),
        })
    });
    pub(crate) static ref SONG_SEARCH: AsyncOnce<Arc<RwLock<SongSearch>>> = AsyncOnce::new(async {
        Arc::new(RwLock::new(
            SongSearch::load(&mut (*DB.get().await).db.try_acquire().unwrap()).await,
        ))
    });
    pub(crate) static ref DOWNLOAD_CACHE: Arc<Mutex<DownloadCache>> =
        Arc::new(Mutex::new(DownloadCache::default()));
}

struct Database {
    db: Pool<Postgres>,
}

struct Ws;

impl Actor for Ws {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Ws {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

#[get("/next")]
async fn next_song(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, ActixError> {
    let resp = ws::start(Ws {}, &req, stream);
    println!("{:?}", resp);
    resp
}

#[get("/prev")]
async fn previous_song(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, ActixError> {
    let resp = ws::start(Ws {}, &req, stream);
    println!("{:?}", resp);
    resp
}

#[get("/playpause")]
async fn play_pause(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, ActixError> {
    let resp = ws::start(Ws {}, &req, stream);
    println!("{:?}", resp);
    resp
}

fn ws_routes() -> Scope {
    web::scope("/ws")
        .service(next_song)
        .service(previous_song)
        .service(play_pause)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    pretty_env_logger::init();

    HttpServer::new(move || {
        let auth0_config = extractors::Auth0Config::default();
        let cors = Cors::permissive();
        App::new()
            .app_data(auth0_config)
            .app_data(&*DB)
            .wrap(cors)
            .wrap(middlewares::err_handlers())
            .wrap(middlewares::security_headers())
            .wrap(middlewares::logger())
            .service(ws_routes())
            .service(api::routes::general_routes())
            .service(api::users::routes())
    })
    .bind((&*CONFIG.host, CONFIG.port))?
    .run()
    .await
}

impl Database {
    pub(crate) async fn setup(uri: &str, timeout: u64) -> Result<Pool<Postgres>, Box<dyn Error>> {
        Ok(Self::try_connect(uri, timeout).await)
    }

    pub async fn try_connect(uri: &str, timeout: u64) -> Pool<Postgres> {
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

    async fn connect(uri: &str) -> Result<Pool<Postgres>, Box<dyn Error>> {
        Ok(Pool::<Postgres>::connect(uri).await?)
    }
}
