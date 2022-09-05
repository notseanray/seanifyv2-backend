mod api;
mod extractors;
mod middlewares;
mod types;

use actix_web::{App, HttpServer};
use dotenv::dotenv;
use actix_cors::Cors;

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
