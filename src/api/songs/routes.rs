use super::handlers;
use actix_web::{web, Scope};

pub fn routes() -> Scope {
    web::scope("/songs")
        .service(handlers::song_new)
        .service(handlers::clear_cache)
        .service(handlers::song_get_data)
        .service(handlers::song_delete_path)
        .service(handlers::song_delete)
        .service(handlers::song_like)
        .service(handlers::song_dislike)
        .service(handlers::song_search)
}
