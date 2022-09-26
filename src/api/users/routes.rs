use super::handlers;
use actix_web::{web, Scope};

pub fn routes() -> Scope {
    web::scope("/users")
        .service(handlers::admin)
        .service(handlers::protected)
        .service(handlers::public)
        .service(handlers::user_taken)
        .service(handlers::user_new)
}
