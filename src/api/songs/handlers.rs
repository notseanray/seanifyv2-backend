use crate::extractors::Claims;
use actix_web::HttpRequest;
use actix_web::{
    get,
    web::{self, Data},
    Responder,
};
use async_once::AsyncOnce;

use sqlx::{query, query_as};

#[get("/new")]
pub async fn new(claims: Claims) -> impl Responder {
    // response!(format!("admin message {}", claims.sub))
    "test".to_string()
}

#[get("/{song}/delete")]
pub async fn new(claims: Claims, song: ) -> impl Responder {
    // response!(format!("admin message {}", claims.sub))
    "test".to_string()
}

#[get("/delete")]
pub async fn new(claims: Claims) -> impl Responder {
    // response!(format!("admin message {}", claims.sub))
    "test".to_string()
}
