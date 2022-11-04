use super::handlers;
use actix_web::{web, Scope};

pub fn routes() -> Scope {
    web::scope("/playlist")
        .service(handlers::playlist_new)
        .service(handlers::playlist_user_data)
        .service(handlers::playlist_hash)
        .service(handlers::playlist_data)
        .service(handlers::playlist_like)
        .service(handlers::playlist_dislike)
        .service(handlers::playlist_add)
        .service(handlers::playlist_delete)
}
