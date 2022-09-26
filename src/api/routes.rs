use super::users;
use actix_web::Scope;

pub fn users_routes() -> Scope {
    users::routes()
}
