use crate::extractors::Claims;
use crate::fetch_db;
use crate::types::Song;
use crate::types::UserFromDB;
use crate::DB;
use actix_web::{
    get,
    web::{self, Data},
    Responder,
};
use actix_web::{HttpRequest, HttpResponse};
use async_once::AsyncOnce;
use web::Path;

use sqlx::{query, query_as};

#[get("/new")]
pub async fn song_new(req: HttpRequest, claims: Claims) -> impl Responder {
    let mut db = fetch_db!();
    if let Some(url) = req.headers().get("url") {
        if let (Some(user), Ok(url)) = (
            UserFromDB::from_id(&mut db, &claims.sub).await,
            url.to_str(),
        ) {
            let song = Song::from_url(url, &mut db).await;
            if song.is_some() {
                return HttpResponse::Ok();
            }
        }
    }
    HttpResponse::BadRequest()
}

#[get("/{song}/delete")]
pub async fn song_delete(claims: Claims, song: Path<String>) -> impl Responder {
    let mut db = fetch_db!();
    if let Some(url) = req.headers().get("url") {
        if let (Some(user), Ok(url)) = (
            UserFromDB::from_id(&mut db, &claims.sub).await,
            url.to_str(),
        ) {
            if user.admin {
            } else {
                query_as!("delete from songs where added_by == $1 and title == $2 ")
            }
        }
    }
    HttpResponse::BadRequest()
}

#[get("/{song}/like")]
pub async fn song_like(claims: Claims) -> impl Responder {
    // response!(format!("admin message {}", claims.sub))
    "test".to_string()
}

#[get("/{song}/dislike")]
pub async fn song_dislike(claims: Claims) -> impl Responder {
    // response!(format!("admin message {}", claims.sub))
    "test".to_string()
}

#[get("/list")]
pub async fn song_list(claims: Claims) -> impl Responder {
    // response!(format!("admin message {}", claims.sub))
    "test".to_string()
}

#[get("/search")]
pub async fn song_search(claims: Claims) -> impl Responder {
    // response!(format!("admin message {}", claims.sub))
    "test".to_string()
}
