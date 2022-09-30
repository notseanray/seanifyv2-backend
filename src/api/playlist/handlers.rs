use crate::extractors::Claims;
use actix_web::HttpRequest;
use actix_web::{
    get,
    web::{self, Data},
    Responder,
};

use sqlx::{query, query_as};

#[get("/new")]
pub async fn playlist_new(claims: Claims) -> impl Responder {
    // response!(format!("admin message {}", claims.sub))
    "test".to_string()
}

#[get("/delete")]
pub async fn playlist_delete(claims: Claims) -> impl Responder {
    // response!(format!("admin message {}", claims.sub))
    "test".to_string()
}

#[get("/{username}")]
pub async fn playlist(username: web::Path<String>, claims: Claims) -> impl Responder {
    // response!(format!("admin message {}", claims.sub))
    "test".to_string()
}

#[get("/{username}/data")]
pub async fn playlist_user_data(username: web::Path<String>, claims: Claims) -> impl Responder {
    // response!(format!("admin message {}", claims.sub))
    "test".to_string()
}

#[get("/{username}/{playlist_name}/hash")]
pub async fn playlist_hash(
    username: web::Path<String>,
    playlist_name: web::Path<String>,
    claims: Claims,
) -> impl Responder {
    // response!(format!("admin message {}", claims.sub))
    "test".to_string()
}

#[get("/{username}/{playlist_name}/data")]
pub async fn playlist_data(
    username: web::Path<String>,
    playlist_name: web::Path<String>,
    claims: Claims,
) -> impl Responder {
    // response!(format!("admin message {}", claims.sub))
    "test".to_string()
}
