use crate::extractors::Claims;
use crate::fetch_db;
use crate::fuzzy::SearchType;
use crate::types::Song;
use crate::types::UserFromDB;
use crate::DB;
use crate::DOWNLOAD_CACHE;
use crate::SONG_SEARCH;
use actix_web::{get, web, Responder};
use actix_web::{HttpRequest, HttpResponse};
use web::Path;

use sqlx::query;

#[get("/new")]
pub async fn song_new(req: HttpRequest, claims: Claims) -> impl Responder {
    let mut db = fetch_db!();
    if let Some(url) = req.headers().get("url") {
        if let (Some(user), Ok(url)) = (
            UserFromDB::from_id(&mut db, &claims.sub).await,
            url.to_str(),
        ) {
            DOWNLOAD_CACHE
                .lock()
                .unwrap()
                .append(url.to_string(), user.id);
            // let song = Song::from_url(url, &mut db, user.id).await;
            // if let Some(_v) = song {
            //     // TODO return JSON?
            //     return HttpResponse::Ok();
            // }
            return HttpResponse::Ok();
        }
    }
    HttpResponse::BadRequest()
}

#[get("/clear_cache")]
pub async fn clear_cache(claims: Claims) -> impl Responder {
    let mut db = fetch_db!();
    if let Some(user) = UserFromDB::from_id(&mut db, &claims.sub).await {
        if !user.admin {
            return HttpResponse::BadRequest();
        }
        DOWNLOAD_CACHE.lock().unwrap().clear();
    }
    HttpResponse::BadRequest()
}

#[get("/{song}/delete")]
pub async fn song_delete_path(claims: Claims, song: Path<String>) -> impl Responder {
    let mut db = fetch_db!();
    let song = song.to_string();
    if let Some(user) = UserFromDB::from_id(&mut db, &claims.sub).await {
        if user.admin {
            if query!("delete from songs where id == $1", song)
                .execute(&mut db)
                .await
                .is_ok()
            {
                return HttpResponse::Ok();
            }
        } else {
            if query!(
                "delete from songs where added_by == $1 and id == $2",
                claims.sub,
                song,
            )
            .execute(&mut db)
            .await
            .is_ok()
            {
                return HttpResponse::Ok();
            }
            return HttpResponse::InternalServerError();
        }
    }
    HttpResponse::BadRequest()
}

#[get("/delete")]
pub async fn song_delete(claims: Claims, req: HttpRequest) -> impl Responder {
    let mut db = fetch_db!();
    if let Some(url) = req.headers().get("url") {
        if let (Some(user), Ok(url)) = (
            UserFromDB::from_id(&mut db, &claims.sub).await,
            url.to_str(),
        ) {
            if user.admin {
                if query!("delete from songs where url == $1", url)
                    .execute(&mut db)
                    .await
                    .is_ok()
                {
                    return HttpResponse::Ok();
                }
            } else {
                if query!(
                    "delete from songs where added_by == $1 and url == $2",
                    claims.sub,
                    url
                )
                .execute(&mut db)
                .await
                .is_ok()
                {
                    return HttpResponse::Ok();
                }
                return HttpResponse::InternalServerError();
            }
        }
        return HttpResponse::BadRequest();
    }
    if let Some(title) = req.headers().get("title") {
        if let (Some(user), Ok(title)) = (
            UserFromDB::from_id(&mut db, &claims.sub).await,
            title.to_str(),
        ) {
            if user.admin {
                if query!("delete from songs where title == $1", title)
                    .execute(&mut db)
                    .await
                    .is_ok()
                {
                    return HttpResponse::Ok();
                }
            } else {
                if query!(
                    "delete from songs where added_by == $1 and title == $2",
                    claims.sub,
                    title
                )
                .execute(&mut db)
                .await
                .is_ok()
                {
                    return HttpResponse::Ok();
                }
                return HttpResponse::InternalServerError();
            }
        }
        return HttpResponse::BadRequest();
    }
    HttpResponse::BadRequest()
}

#[get("/{song}/like")]
pub async fn song_like(claims: Claims, song: Path<String>) -> impl Responder {
    let mut db = fetch_db!();
    if let Some(mut u) = UserFromDB::from_id(&mut db, &claims.sub).await {
        u.like(song.as_str());
        HttpResponse::Ok()
    } else {
        HttpResponse::BadRequest()
    }
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
pub async fn song_search(claims: Claims, req: HttpRequest) -> impl Responder {
    let search_type = if let Some(search_type) = req.headers().get("search_type") {
        match search_type.to_str().unwrap_or_default() {
            "uploader" => SearchType::Uploader,
            "title" => SearchType::Title,
            "user" => SearchType::User,
            "id" => SearchType::Id,
            _ => SearchType::Default,
        }
    } else {
        SearchType::Default
    };
    let search_count = if let Some(search_count) = req.headers().get("search_count") {
        search_count
            .to_str()
            .unwrap_or_default()
            .parse::<usize>()
            .unwrap_or(30)
    } else {
        30
    };
    let mut db = fetch_db!();
    if let Some(search_term) = req.headers().get("search") {
        if let (Some(_user), Ok(search_term)) = (
            UserFromDB::from_id(&mut db, &claims.sub).await,
            search_term.to_str(),
        ) {
            if let SearchType::Id = search_type {
                return match &SONG_SEARCH
                    .get()
                    .await
                    .write()
                    .unwrap()
                    .get_by_id(search_term)
                {
                    Some(v) => serde_json::to_string(v).unwrap(),
                    _ => String::from("{}"),
                };
            }
            return serde_json::to_string(&SONG_SEARCH.get().await.write().unwrap().search(
                search_term,
                search_type,
                search_count,
            ))
            .unwrap();
        }
    }
    String::from("{}")
}
