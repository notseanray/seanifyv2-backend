use super::users;
use crate::{Database, BRANCH, VERSION};
use std::time::{UNIX_EPOCH, SystemTime, Duration};
use actix_web::Scope;
use actix_web::{
    get,
    web::{self, Data},
    Responder,
};
use crate::response;
use crate::api::types::{Metadata, Message};

#[get("/ping")]
pub async fn ping() -> impl Responder {
    response!(format!(
        "{:?}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis()
    ))
}
pub fn general_routes() -> Scope {
    web::scope("/")
        .service(ping)
}
