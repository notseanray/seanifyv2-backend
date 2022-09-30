use super::users;
use crate::api::types::{Message, Metadata};
use crate::response;
use crate::{time, Database, BRANCH, VERSION};
use actix_web::Scope;
use actix_web::{
    get,
    web::{self, Data},
    Responder,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[get("/ping")]
pub async fn ping() -> impl Responder {
    response!(format!("{:?}", time!()))
}
pub fn general_routes() -> Scope {
    web::scope("/").service(ping)
}
