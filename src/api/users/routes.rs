use super::handlers;
use actix_web::{web, Scope};

pub fn routes() -> Scope {
    web::scope("/users")
        .service(handlers::user_taken)
        .service(handlers::user_new)
        .service(handlers::user_self)
        .service(handlers::edit)
        .service(handlers::follow)
        .service(handlers::unfollow)
        .service(handlers::delete)
        .service(handlers::delete_user)
        .service(handlers::listen)
        .service(handlers::get_user_from_id)
        .service(handlers::get_user_from_name)
}
