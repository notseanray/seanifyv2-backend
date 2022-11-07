use crate::extractors::Claims;
use crate::types::UserFromDB;
use crate::types::{Playlist, PlaylistDB};
use crate::DB;
use web::Path;
use crate::{fetch_db, time};
use actix_web::{get, web, Responder};
use actix_web::{HttpRequest, HttpResponse};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use sqlx::{query, query_as};

#[get("/new")]
pub async fn playlist_new(claims: Claims, req: HttpRequest) -> impl Responder {
    if let (Some(v), Some(u)) = (
        req.headers().get("data"),
        UserFromDB::from_id(&mut fetch_db!(), &claims.sub).await,
    ) {
        // TODO ERROR HANDLING
        let mut data: Playlist = serde_json::from_str(v.to_str().unwrap()).unwrap();
        data.author_id = claims.sub;
        // get username from id
        data.author = u.username;
        data.description.truncate(400);
        data.name.truncate(100);
        data.likes.clear();
        data.duration = 0;
        data.lastupdate = time!();
        data.cover.truncate(2000);
        HttpResponse::Ok()
    } else {
        HttpResponse::BadRequest()
    }
}

#[get("/{username}")]
pub async fn playlist_user_data(username: Path<String>, claims: Claims) -> impl Responder {
    let username = username.to_string();
    // if they have an account
    let mut db = fetch_db!();
    if let Some(u) = UserFromDB::from_id(&mut db, &claims.sub).await {
        let playlist = query_as!(
            PlaylistDB,
            "select * from playlist where author == $1",
            username,
        )
        .fetch_all(&mut db)
        .await;
        if let Ok(v) = playlist {
            let playlist: Vec<Playlist> = v
                .into_iter()
                .filter_map(|x| {
                    let x: Playlist = x.into();
                    if x.author_id == claims.sub
                        || x.edit_list.contains(&username)
                        || x.public_playlist
                        || u.admin
                    {
                        Some(x)
                    } else {
                        None
                    }
                })
                .collect();
            if let Ok(v) = serde_json::to_string(&playlist) {
                return v;
            }
        }
    }
    "[]".to_string()
}

#[get("/{username}/{playlist_name}/hash")]
pub async fn playlist_hash(
    username: Path<String>,
    playlist_name: Path<String>,
    claims: Claims,
) -> impl Responder {
    let mut db = fetch_db!();
    let username = username.to_string();
    if let Some(u) = UserFromDB::from_id(&mut db, &claims.sub).await {
        let playlist_name = playlist_name.to_string();
        // if they are requesting themself
        let playlist = query_as!(
            PlaylistDB,
            "select * from playlist where author == $1 and name == $2",
            username,
            playlist_name,
        )
        .fetch_all(&mut db)
        .await;
        if let Ok(v) = playlist {
            let v: Vec<Playlist> = v
                .into_iter()
                .filter_map(|x| {
                    let x: Playlist = x.into();
                    if x.author_id == claims.sub
                        || x.edit_list.contains(&username)
                        || x.public_playlist
                        || u.admin
                    {
                        Some(x)
                    } else {
                        None
                    }
                })
                .collect();
            let mut hasher = blake3::Hasher::new();
            for ele in v {
                hasher.update(ele.songs.join("").as_bytes());
            }
            return hasher.finalize().to_string();
        }
    }
    "".to_string()
}

#[get("/{username}/{playlist_name}/data")]
pub async fn playlist_data(
    username: Path<String>,
    playlist_name: Path<String>,
    claims: Claims,
) -> impl Responder {
    let mut db = fetch_db!();
    if let Some(u) = UserFromDB::from_id(&mut db, &claims.sub).await {
        let playlist_name = playlist_name.to_string();
        // if they are requesting themself
        let playlist = query_as!(
            PlaylistDB,
            "select * from playlist where author == $1 and name == $2",
            u.username,
            playlist_name
        )
        .fetch_all(&mut db)
        .await;
        if let Ok(v) = playlist {
            let playlist: Vec<Playlist> = v
                .into_iter()
                .filter_map(|x| {
                    let x: Playlist = x.into();
                    if x.author_id == claims.sub
                        || x.edit_list.contains(&username)
                        || x.public_playlist
                    {
                        Some(x)
                    } else {
                        None
                    }
                })
                .collect();
            if let Ok(v) = serde_json::to_string(&playlist) {
                return v;
            }
        }
    }
    "[]".to_string()
}

#[get("/{username}/{playlist_name}/like")]
pub async fn playlist_like(
    _username: Path<String>,
    playlist_name: Path<String>,
    claims: Claims,
) -> impl Responder {
    let playlist_name = playlist_name.to_string();
    let mut db = fetch_db!();
    if let Some(u) = UserFromDB::from_id(&mut db, &claims.sub).await {
        // if they are requesting themself
        let playlist = query_as!(
            PlaylistDB,
            "select * from playlist where author == $1 and name == $2",
            u.username,
            playlist_name,
        )
        .fetch_optional(&mut db)
        .await;
        if let Ok(Some(v)) = playlist {
            let followers = PlaylistDB::like(&v.likes, &claims.sub);
            let v: Playlist = v.into();
            if !(v.author_id == claims.sub
                || v.edit_list.contains(&claims.sub)
                || v.public_playlist
                || u.admin)
            {
                return HttpResponse::Forbidden();
            }
            if query!(
                "update playlist set likes = $1 where author == $2 and name == $3",
                followers,
                u.username,
                playlist_name
            )
            .execute(&mut db)
            .await
            .is_ok()
            {
                return HttpResponse::Ok();
            }
        }
    }
    HttpResponse::BadRequest()
}

#[get("/{username}/{playlist_name}/dislike")]
pub async fn playlist_dislike(
    _username: Path<String>,
    playlist_name: Path<String>,
    claims: Claims,
) -> impl Responder {
    let playlist_name = playlist_name.to_string();
    let mut db = fetch_db!();
    if let Some(u) = UserFromDB::from_id(&mut db, &claims.sub).await {
        // if they are requesting themself
        let playlist = query_as!(
            PlaylistDB,
            "select * from playlist where author == $1 and name == $2",
            u.username,
            playlist_name,
        )
        .fetch_optional(&mut db)
        .await;
        if let Ok(Some(v)) = playlist {
            let followers = PlaylistDB::dislike(&v.likes, &claims.sub);
            if query!(
                "update playlist set likes = $1 where author == $2 and name == $3",
                followers,
                u.username,
                playlist_name
            )
            .execute(&mut db)
            .await
            .is_ok()
            {
                if !(v.author_id == claims.sub
                    || v.edit_list.contains(&claims.sub)
                    || v.public_playlist
                    || u.admin)
                {
                    return HttpResponse::Forbidden();
                } else {
                    return HttpResponse::Ok();
                }
            }
        }
    }
    HttpResponse::BadRequest()
}

#[get("/{username}/{playlist_name}/add")]
pub async fn playlist_add(
    _username: Path<String>,
    playlist_name: Path<String>,
    claims: Claims,
) -> impl Responder {
    let playlist_name = playlist_name.to_string();
    let mut db = fetch_db!();
    if let Some(u) = UserFromDB::from_id(&mut db, &claims.sub).await {
        // if they are requesting themself
        let playlist = query_as!(
            PlaylistDB,
            "select * from playlist where author == $1 and name == $2",
            u.username,
            playlist_name,
        )
        .fetch_optional(&mut db)
        .await;
        if let Ok(Some(v)) = playlist {
            let followers = PlaylistDB::dislike(&v.likes, &claims.sub);
            if !(v.author_id == claims.sub
                || v.edit_list.contains(&claims.sub)
                || v.public_playlist
                || u.admin)
            {
                return HttpResponse::Forbidden();
            }
            if query!(
                "update playlist set likes = $1 where author == $2 and name == $3",
                followers,
                u.username,
                playlist_name
            )
            .execute(&mut db)
            .await
            .is_ok()
            {
                return HttpResponse::Ok();
            }
        }
    }
    HttpResponse::BadRequest()
}

#[get("/{username}/{playlist_name}/delete")]
pub async fn playlist_delete(
    _username: Path<String>,
    playlist_name: Path<String>,
    claims: Claims,
) -> impl Responder {
    let playlist_name = playlist_name.to_string();
    let mut db = fetch_db!();
    if let Some(u) = UserFromDB::from_id(&mut db, &claims.sub).await {
        // if they are requesting themself
        let playlist = query_as!(
            PlaylistDB,
            "select * from playlist where author == $1 and name == $2",
            u.username,
            playlist_name,
        )
        .fetch_optional(&mut db)
        .await;
        if let Ok(Some(v)) = playlist {
            if v.author_id == claims.sub || u.admin {
                return match query!(
                    "delete from playlist where author == $1 and name == $2",
                    u.username,
                    playlist_name
                )
                .execute(&mut db)
                .await
                {
                    Ok(_) => HttpResponse::Ok(),
                    _ => HttpResponse::Forbidden(),
                };
            }
        }
    }
    HttpResponse::BadRequest()
}
