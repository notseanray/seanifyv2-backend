use crate::extractors::Claims;
use crate::fetch_db;
use crate::fuzzy::SearchType;
use crate::types::Song;
use crate::types::User;
use crate::types::MAX_SEARCH_RESULTS;
use crate::DB;
use crate::DOWNLOAD_CACHE;
use crate::SONG_SEARCH;
use actix_web::{get, web, Responder};
use actix_web::{HttpRequest, HttpResponse};
use sqlx::query_as;
use std::sync::Arc;
use web::Path;

use sqlx::query;

#[get("/new")]
pub async fn song_new(req: HttpRequest, claims: Claims) -> impl Responder {
    let mut db = fetch_db!();
    let Some(url) = req.headers().get("url") else {
        return HttpResponse::BadRequest();
    };
    let (Some(user), Ok(url)) = (
        User::from_id(&mut db, &claims.sub).await,
        url.to_str(),
    ) else {
        return HttpResponse::BadRequest();
    };
    DOWNLOAD_CACHE
        .lock()
        .unwrap()
        .append(url.to_string(), user.id);
    // let song = Song::from_url(url, &mut db, user.id).await;
    // if let Some(_v) = song {
    //     // TODO return JSON?
    //     return HttpResponse::Ok();
    // }
    HttpResponse::Ok()
}

#[get("/clear_cache")]
pub async fn clear_cache(claims: Claims) -> impl Responder {
    let mut db = fetch_db!();
    let Some(user) = User::from_id(&mut db, &claims.sub).await else {
        return HttpResponse::Unauthorized();
    };
    if !user.admin {
        return HttpResponse::Forbidden();
    }
    DOWNLOAD_CACHE.lock().unwrap().clear();
    HttpResponse::Ok()
}

#[get("/{song}")]
pub async fn song_get_data(claims: Claims, song: Path<String>) -> impl Responder {
    let mut db = fetch_db!();
    let song = song.to_string();
    let Some(_u) = User::from_id(&mut db, &claims.sub).await else {
        return "".into();
    };
    let Ok(song) = query_as!(Song, "select * from songs where id = $1", song).fetch_optional(&mut db).await else {
        return "{}".into();
    };
    if let Ok(v) = serde_json::to_string(&song) {
        v
    } else {
        "".into()
    }
}

#[get("/{song}/delete")]
pub async fn song_delete_path(claims: Claims, song: Path<String>) -> impl Responder {
    let mut db = fetch_db!();
    let song = song.to_string();
    let Some(user) = User::from_id(&mut db, &claims.sub).await else {
        return HttpResponse::Unauthorized();
    };
    if user.admin {
        if query!("delete from songs where id = $1", song)
            .execute(&mut db)
            .await
            .is_ok()
        {
            return HttpResponse::Ok();
        }
    } else if query!(
        "delete from songs where added_by = $1 and id = $2",
        claims.sub,
        song,
    )
    .execute(&mut db)
    .await
    .is_ok()
    {
        return HttpResponse::Ok();
    }
    HttpResponse::InternalServerError()
}

#[get("/delete")]
pub async fn song_delete(claims: Claims, req: HttpRequest) -> impl Responder {
    let mut db = fetch_db!();
    if let Some(url) = req.headers().get("url") {
        let (Some(user), Ok(url)) = (
            User::from_id(&mut db, &claims.sub).await,
            url.to_str(),
        ) else {
            return HttpResponse::BadRequest();
        };
        if user.admin {
            if query!("delete from songs where url = $1", url)
                .execute(&mut db)
                .await
                .is_ok()
            {
                return HttpResponse::Ok();
            }
        } else if query!(
            "delete from songs where added_by = $1 and url = $2",
            claims.sub,
            url
        )
        .execute(&mut db)
        .await
        .is_ok()
        {
            return HttpResponse::Ok();
        }
        return HttpResponse::BadRequest();
    }
    let Some(title) = req.headers().get("title") else {
        return HttpResponse::BadRequest();
    };
    let (Some(user), Ok(title)) = (
        User::from_id(&mut db, &claims.sub).await,
        title.to_str(),
    ) else {
        return HttpResponse::Unauthorized();
    };
    if user.admin {
        if query!("delete from songs where title = $1", title)
            .execute(&mut db)
            .await
            .is_ok()
        {
            return HttpResponse::Ok();
        }
    } else if query!(
        "delete from songs where added_by = $1 and title = $2",
        claims.sub,
        title
    )
    .execute(&mut db)
    .await
    .is_ok()
    {
        return HttpResponse::Ok();
    }
    HttpResponse::BadRequest()
}

#[get("/{song}/like")]
pub async fn song_like(claims: Claims, song: Path<String>) -> impl Responder {
    let mut db = fetch_db!();
    let Some(mut u) = User::from_id(&mut db, &claims.sub).await else {
        return HttpResponse::BadRequest();
    };
    u.like(song.to_string());
    // query!("update users where id = $1");
    HttpResponse::Ok()
}

#[get("/{song}/dislike")]
pub async fn song_dislike(claims: Claims, song: Path<String>) -> impl Responder {
    let mut db = fetch_db!();
    let Some(mut u) = User::from_id(&mut db, &claims.sub).await else {
        return HttpResponse::BadRequest();
    };
    u.dislike(song.as_str());
    HttpResponse::Ok()
}

// #[get("/list")]
// pub async fn song_list(claims: Claims) -> impl Responder {
//     let mut db = fetch_db!();
//     let Some(_u) = UserFromDB::from_id(&mut db, &claims.sub).await else {
//         unreachable!();
//     };
//     let songs = query_as!(Song, "select * from songs")
//         .fetch_all(&mut db)
//         .await
//         .unwrap();
//     // TODO STREAM RESPONSE
//     let response = songs.into_iter().map(|x| serde_json::to_string(&x).unwrap_or_default().as_bytes());
//     HttpResponseBuilder::new(StatusCode::OK).streaming::<u8, std::io::Error>(response.into())
// }

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
            .unwrap_or(MAX_SEARCH_RESULTS)
    } else {
        MAX_SEARCH_RESULTS
    };
    let mut db = fetch_db!();
    if let Some(search_term) = req.headers().get("search") {
        let (Some(_user), Ok(search_term)) = (
            User::from_id(&mut db, &claims.sub).await,
            search_term.to_str(),
        ) else {
            return String::from("{}");
        };
        // remove get by id
        if let SearchType::Id = search_type {
            let Ok(song) = query_as!(Song, "select * from songs where id = $1", search_term).fetch_optional(&mut db).await else {
                return "{}".into();
            };
            return if let Ok(v) = serde_json::to_string(&song) {
                v
            } else {
                "".into()
            };
        }
        let search_term = Arc::new(search_term.to_string());
        let res = tokio::spawn(async move {
            let search_term = search_term.clone();
            serde_json::to_string(&SONG_SEARCH.get().await.write().await.search(
                &search_term,
                search_type,
                search_count,
            ))
            .ok()
        })
        .await;
        return if let Ok(Some(v)) = res {
            v
        } else {
            "[]".to_string()
        };
    }
    String::from("[]")
}
